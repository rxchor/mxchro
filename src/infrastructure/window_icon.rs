//! Owned, per-window Windows icons. Never alters class icons, EXEs or shortcuts.
use crate::application::ports::{WindowIconBackend, WindowIconError};
use resvg::{tiny_skia, usvg};
use std::{ffi::c_void, marker::PhantomData, rc::Rc};
use windows::Win32::{
    Foundation::{HWND, LPARAM, WPARAM},
    Graphics::Gdi::{
        BI_RGB, BITMAPINFO, BITMAPINFOHEADER, CreateBitmap, CreateDIBSection, DIB_RGB_COLORS,
        DeleteObject, HBITMAP,
    },
    System::Threading::GetCurrentThreadId,
    UI::{
        HiDpi::{GetDpiForWindow, GetSystemMetricsForDpi},
        WindowsAndMessaging::{
            CreateIconIndirect, DestroyIcon, GetWindowThreadProcessId, HICON, ICON_BIG, ICON_SMALL,
            ICONINFO, IsWindow, SM_CXSMICON, SendMessageW, WM_GETICON, WM_SETICON,
        },
    },
};

struct OwnedIcon(HICON);
impl Drop for OwnedIcon {
    fn drop(&mut self) {
        // Only handles created by CreateIconIndirect are owned here.
        unsafe {
            let _ = DestroyIcon(self.0);
        }
    }
}

struct OwnedBitmap(HBITMAP);
impl Drop for OwnedBitmap {
    fn drop(&mut self) {
        unsafe {
            let _ = DeleteObject(self.0.into());
        }
    }
}

struct IconPair {
    small: OwnedIcon,
    large: OwnedIcon,
}

pub struct WindowsWindowIcon {
    window: HWND,
    icons: Option<IconPair>,
    previous: Option<(HICON, HICON)>,
    // Window messages and ownership changes must stay on the owning UI thread.
    _ui_thread: PhantomData<Rc<()>>,
}

impl WindowsWindowIcon {
    pub fn new(native_window: isize) -> Self {
        Self {
            window: HWND(native_window as *mut c_void),
            icons: None,
            previous: None,
            _ui_thread: PhantomData,
        }
    }

    fn owns_current_thread(&self) -> bool {
        unsafe {
            IsWindow(Some(self.window)).as_bool()
                && GetWindowThreadProcessId(self.window, None) == GetCurrentThreadId()
        }
    }
}

impl WindowIconBackend for WindowsWindowIcon {
    fn set_icon(&mut self, svg: &[u8]) -> Result<(), WindowIconError> {
        if !self.owns_current_thread() {
            return Err(WindowIconError::Native);
        }
        let tree = usvg::Tree::from_data(svg, &usvg::Options::default()).map_err(|error| {
            log::warn!("Could not parse window icon SVG: {error}");
            WindowIconError::InvalidSvg
        })?;
        let dpi = unsafe { GetDpiForWindow(self.window) }.max(96);
        let small_size = unsafe { GetSystemMetricsForDpi(SM_CXSMICON, dpi) }.clamp(16, 256) as u32;
        // Build both first: a failed allocation never replaces the current pair.
        let next = IconPair {
            small: create_icon(&tree, small_size)?,
            large: create_icon(&tree, 256)?,
        };
        unsafe {
            let small = SendMessageW(
                self.window,
                WM_SETICON,
                Some(WPARAM(ICON_SMALL as usize)),
                Some(LPARAM(next.small.0.0 as isize)),
            );
            let large = SendMessageW(
                self.window,
                WM_SETICON,
                Some(WPARAM(ICON_BIG as usize)),
                Some(LPARAM(next.large.0.0 as isize)),
            );
            self.previous
                .get_or_insert((HICON(small.0 as *mut c_void), HICON(large.0 as *mut c_void)));
        }
        // Old handles are released only after neither is installed on the window.
        self.icons = Some(next);
        Ok(())
    }
}

impl Drop for WindowsWindowIcon {
    fn drop(&mut self) {
        if self.owns_current_thread()
            && let (Some(icons), Some((small, large))) = (&self.icons, self.previous)
        {
            unsafe {
                // Do not overwrite another owner's icon, or a recycled HWND.
                if current_icon(self.window, ICON_SMALL) == icons.small.0 {
                    SendMessageW(
                        self.window,
                        WM_SETICON,
                        Some(WPARAM(ICON_SMALL as usize)),
                        Some(LPARAM(small.0 as isize)),
                    );
                }
                if current_icon(self.window, ICON_BIG) == icons.large.0 {
                    SendMessageW(
                        self.window,
                        WM_SETICON,
                        Some(WPARAM(ICON_BIG as usize)),
                        Some(LPARAM(large.0 as isize)),
                    );
                }
            }
        }
        // The fields release only our icons, never the borrowed original handles.
    }
}

unsafe fn current_icon(window: HWND, kind: u32) -> HICON {
    let value = unsafe { SendMessageW(window, WM_GETICON, Some(WPARAM(kind as usize)), None) };
    HICON(value.0 as *mut c_void)
}

fn create_icon(tree: &usvg::Tree, size: u32) -> Result<OwnedIcon, WindowIconError> {
    let mut pixmap = tiny_skia::Pixmap::new(size, size).ok_or(WindowIconError::Native)?;
    let scale = (size as f32 / tree.size().width()).min(size as f32 / tree.size().height());
    let transform = tiny_skia::Transform::from_row(
        scale,
        0.,
        0.,
        scale,
        (size as f32 - tree.size().width() * scale) / 2.,
        (size as f32 - tree.size().height() * scale) / 2.,
    );
    resvg::render(tree, transform, &mut pixmap.as_mut());
    let (bgra, mask) = icon_pixels(pixmap.data(), size);
    let info = BITMAPINFO {
        bmiHeader: BITMAPINFOHEADER {
            biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
            biWidth: size as i32,
            biHeight: -(size as i32),
            biPlanes: 1,
            biBitCount: 32,
            biCompression: BI_RGB.0,
            ..Default::default()
        },
        ..Default::default()
    };
    let native_error = |error: windows::core::Error| {
        log::warn!("Could not create Windows icon: {error}");
        WindowIconError::Native
    };
    unsafe {
        let mut bits = std::ptr::null_mut();
        let color = OwnedBitmap(
            CreateDIBSection(None, &info, DIB_RGB_COLORS, &mut bits, None, 0)
                .map_err(native_error)?,
        );
        if bits.is_null() {
            return Err(WindowIconError::Native);
        }
        // The 32-bit top-down DIB owns size * size * 4 writable bytes.
        std::ptr::copy_nonoverlapping(bgra.as_ptr(), bits.cast::<u8>(), bgra.len());
        let mask = OwnedBitmap(CreateBitmap(
            size as i32,
            size as i32,
            1,
            1,
            Some(mask.as_ptr().cast()),
        ));
        if mask.0.is_invalid() {
            return Err(WindowIconError::Native);
        }
        let icon = CreateIconIndirect(&ICONINFO {
            fIcon: true.into(),
            hbmMask: mask.0,
            hbmColor: color.0,
            ..Default::default()
        })
        .map_err(native_error)?;
        Ok(OwnedIcon(icon))
    }
}

/// Keep premultiplied alpha; Windows DIBs use BGRA and a word-aligned AND mask.
fn icon_pixels(rgba: &[u8], size: u32) -> (Vec<u8>, Vec<u8>) {
    let stride = (size as usize).div_ceil(16) * 2;
    let mut mask = vec![0; stride * size as usize];
    let mut bgra = rgba.to_vec();
    for (index, pixel) in bgra.as_chunks_mut::<4>().0.iter_mut().enumerate() {
        pixel.swap(0, 2);
        if pixel[3] == 0 {
            let (x, y) = (index % size as usize, index / size as usize);
            mask[y * stride + x / 8] |= 0x80 >> (x % 8);
        }
    }
    (bgra, mask)
}

#[cfg(test)]
#[path = "../../tests/unit/infrastructure/window_icon.rs"]
mod tests;
