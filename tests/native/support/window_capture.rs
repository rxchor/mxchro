//! Capture only a visible window belonging to this fixture's process.
//! Local FFI keeps screenshot-only GDI APIs out of production dependencies.
use std::ffi::c_void;
type Handle = *mut c_void;

#[derive(PartialEq, Eq)]
pub struct Capture {
    width: usize,
    height: usize,
    pixels: Vec<u8>,
}

impl Capture {
    /// Rectangles are logical coordinates in the fixture's 1160 x 780 window.
    pub fn same_outside(&self, other: &Self, rectangles: &[(usize, usize, usize, usize)]) -> bool {
        self.width == other.width
            && self.height == other.height
            && self
                .pixels
                .as_chunks::<4>()
                .0
                .iter()
                .zip(other.pixels.as_chunks::<4>().0.iter())
                .enumerate()
                .all(|(i, (a, b))| {
                    let (x, y) = (i % self.width, i / self.width);
                    a == b
                        || rectangles.iter().any(|&(left, top, right, bottom)| {
                            x >= left * self.width / 1160
                                && x < right * self.width / 1160
                                && y >= top * self.height / 780
                                && y < bottom * self.height / 780
                        })
                })
    }
}

#[repr(C)]
#[derive(Default)]
struct Rect {
    left: i32,
    top: i32,
    right: i32,
    bottom: i32,
}
#[repr(C)]
struct BitmapInfo {
    size: u32,
    width: i32,
    height: i32,
    planes: u16,
    bits: u16,
    compression: u32,
    image_size: u32,
    xppm: i32,
    yppm: i32,
    colors_used: u32,
    colors_important: u32,
    color: u32,
}

#[link(name = "user32")]
unsafe extern "system" {
    fn EnumWindows(callback: unsafe extern "system" fn(Handle, isize) -> i32, data: isize) -> i32;
    fn GetWindowThreadProcessId(window: Handle, process: *mut u32) -> u32;
    fn IsWindowVisible(window: Handle) -> i32;
    fn GetClientRect(window: Handle, rect: *mut Rect) -> i32;
    fn GetDC(window: Handle) -> Handle;
    fn ReleaseDC(window: Handle, dc: Handle) -> i32;
    fn PrintWindow(window: Handle, dc: Handle, flags: u32) -> i32;
}
#[link(name = "gdi32")]
unsafe extern "system" {
    fn CreateCompatibleDC(dc: Handle) -> Handle;
    fn CreateCompatibleBitmap(dc: Handle, width: i32, height: i32) -> Handle;
    fn SelectObject(dc: Handle, object: Handle) -> Handle;
    fn GetDIBits(
        dc: Handle,
        bitmap: Handle,
        start: u32,
        lines: u32,
        pixels: *mut c_void,
        info: *mut BitmapInfo,
        usage: u32,
    ) -> i32;
    fn DeleteObject(object: Handle) -> i32;
    fn DeleteDC(dc: Handle) -> i32;
}

unsafe extern "system" fn find_own_window(window: Handle, data: isize) -> i32 {
    let mut pid = 0;
    unsafe {
        GetWindowThreadProcessId(window, &mut pid);
        if pid == std::process::id() && IsWindowVisible(window) != 0 {
            *(data as *mut Handle) = window;
            return 0;
        }
    }
    1
}

pub fn pixels() -> Capture {
    unsafe {
        let mut window: Handle = std::ptr::null_mut();
        EnumWindows(find_own_window, (&mut window as *mut Handle) as isize);
        assert!(!window.is_null(), "fixture window was not found");
        let mut rect = Rect::default();
        assert_ne!(GetClientRect(window, &mut rect), 0);
        let (width, height) = (rect.right - rect.left, rect.bottom - rect.top);
        let dc = GetDC(window);
        let memory = CreateCompatibleDC(dc);
        let bitmap = CreateCompatibleBitmap(dc, width, height);
        assert!(!dc.is_null() && !memory.is_null() && !bitmap.is_null());
        let old = SelectObject(memory, bitmap);
        // Client only + full GPU-rendered content. Never capture another process.
        let printed = PrintWindow(window, memory, 3);
        SelectObject(memory, old);
        let mut info = BitmapInfo {
            size: 40,
            width,
            height: -height,
            planes: 1,
            bits: 32,
            compression: 0,
            image_size: 0,
            xppm: 0,
            yppm: 0,
            colors_used: 0,
            colors_important: 0,
            color: 0,
        };
        let mut pixels = vec![0u8; width as usize * height as usize * 4];
        let lines = GetDIBits(
            memory,
            bitmap,
            0,
            height as u32,
            pixels.as_mut_ptr().cast(),
            &mut info,
            0,
        );
        DeleteObject(bitmap);
        DeleteDC(memory);
        ReleaseDC(window, dc);
        assert_ne!(printed, 0, "PrintWindow failed");
        assert_eq!(lines, height, "GetDIBits failed");
        for pixel in pixels.as_chunks_mut::<4>().0 {
            pixel[3] = 255;
        }
        assert!(
            pixels
                .as_chunks::<4>()
                .0
                .iter()
                .any(|p| p[0] != p[1] || p[1] != p[2] || p[0] > 32),
            "capture appears blank"
        );
        if let Some(directory) = std::env::var_os("MXCHRO_CAPTURE_DIR") {
            let mut bmp = vec![0u8; 54];
            bmp[..2].copy_from_slice(b"BM");
            bmp[2..6].copy_from_slice(&(54 + pixels.len() as u32).to_le_bytes());
            bmp[10..14].copy_from_slice(&54u32.to_le_bytes());
            bmp[14..18].copy_from_slice(&40u32.to_le_bytes());
            bmp[18..22].copy_from_slice(&width.to_le_bytes());
            bmp[22..26].copy_from_slice(&(-height).to_le_bytes());
            bmp[26..28].copy_from_slice(&1u16.to_le_bytes());
            bmp[28..30].copy_from_slice(&32u16.to_le_bytes());
            bmp.extend_from_slice(&pixels);
            static CAPTURE_INDEX: std::sync::atomic::AtomicUsize =
                std::sync::atomic::AtomicUsize::new(0);
            let index = CAPTURE_INDEX.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            std::fs::write(
                std::path::PathBuf::from(directory).join(format!("toggle-feedback-{index}.bmp")),
                bmp,
            )
            .unwrap();
        }
        Capture {
            width: width as usize,
            height: height as usize,
            pixels,
        }
    }
}
