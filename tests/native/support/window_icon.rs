//! Only inspects icons belonging to this fixture's own window.
use gpui_kit::{AssetSource, Window};
use mxchro::{
    application::{
        dto::{LogoChoice, ThemeChoice},
        ports::{WindowIconBackend, WindowIconError},
    },
    infrastructure::window_icon::WindowsWindowIcon,
    view::assets::{AppAssets, logo_path},
};
use raw_window_handle::{HasWindowHandle, RawWindowHandle};
use windows::Win32::{
    Foundation::{HWND, WPARAM},
    Graphics::Gdi::{
        BI_RGB, BITMAP, BITMAPINFO, BITMAPINFOHEADER, CreateCompatibleDC, DIB_RGB_COLORS, DeleteDC,
        DeleteObject, GetDIBits, GetObjectW,
    },
    UI::WindowsAndMessaging::{
        GetIconInfo, HICON, ICON_BIG, ICON_SMALL, ICONINFO, SendMessageW, WM_GETICON,
    },
};

fn native(window: &Window) -> HWND {
    let RawWindowHandle::Win32(handle) = HasWindowHandle::window_handle(window).unwrap().as_raw()
    else {
        panic!("expected Windows fixture");
    };
    HWND(handle.hwnd.get() as *mut _)
}

pub fn backend(window: &Window) -> Box<dyn WindowIconBackend> {
    Box::new(WindowsWindowIcon::new(native(window).0 as isize))
}

pub fn handles(window: &Window) -> (isize, isize) {
    let hwnd = native(window);
    unsafe {
        (
            SendMessageW(hwnd, WM_GETICON, Some(WPARAM(ICON_SMALL as usize)), None).0,
            SendMessageW(hwnd, WM_GETICON, Some(WPARAM(ICON_BIG as usize)), None).0,
        )
    }
}

struct CopiedIcon(ICONINFO);
impl Drop for CopiedIcon {
    fn drop(&mut self) {
        unsafe {
            let _ = DeleteObject(self.0.hbmColor.into());
            let _ = DeleteObject(self.0.hbmMask.into());
        }
    }
}

fn digest(icon: isize) -> u64 {
    assert_ne!(icon, 0);
    unsafe {
        let mut info = ICONINFO::default();
        GetIconInfo(HICON(icon as *mut _), &mut info).unwrap();
        let info = CopiedIcon(info);
        let mut bitmap = BITMAP::default();
        assert_ne!(
            GetObjectW(
                info.0.hbmColor.into(),
                std::mem::size_of::<BITMAP>() as i32,
                Some((&mut bitmap as *mut BITMAP).cast())
            ),
            0
        );
        let mut header = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: bitmap.bmWidth,
                biHeight: -bitmap.bmHeight.abs(),
                biPlanes: 1,
                biBitCount: 32,
                biCompression: BI_RGB.0,
                ..Default::default()
            },
            ..Default::default()
        };
        let mut pixels =
            vec![0u8; bitmap.bmWidth as usize * bitmap.bmHeight.unsigned_abs() as usize * 4];
        let dc = CreateCompatibleDC(None);
        assert!(!dc.is_invalid());
        let lines = GetDIBits(
            dc,
            info.0.hbmColor,
            0,
            bitmap.bmHeight.unsigned_abs(),
            Some(pixels.as_mut_ptr().cast()),
            &mut header,
            DIB_RGB_COLORS,
        );
        let _ = DeleteDC(dc);
        assert_eq!(lines, bitmap.bmHeight.abs());
        pixels.iter().fold(0xcbf29ce484222325, |hash, byte| {
            (hash ^ u64::from(*byte)).wrapping_mul(0x100000001b3)
        })
    }
}

pub fn digests(icons: (isize, isize)) -> (u64, u64) {
    (digest(icons.0), digest(icons.1))
}

pub fn assert_released(icons: (isize, isize)) {
    for icon in [icons.0, icons.1] {
        let mut info = ICONINFO::default();
        let valid = unsafe { GetIconInfo(HICON(icon as *mut _), &mut info).is_ok() };
        if valid {
            drop(CopiedIcon(info));
        }
        assert!(!valid, "old HICON was not released");
    }
}

pub fn check_failure_and_restore(window: &Window) {
    let original = handles(window);
    let mut temporary = backend(window);
    assert_eq!(
        temporary.set_icon(b"invalid SVG"),
        Err(WindowIconError::InvalidSvg)
    );
    assert_eq!(
        handles(window),
        original,
        "invalid input replaced the installed icons"
    );
    let svg = AppAssets
        .load(logo_path(LogoChoice::RosePine, ThemeChoice::Monochrome))
        .unwrap()
        .unwrap();
    temporary.set_icon(&svg).unwrap();
    let replaced = handles(window);
    assert_ne!(digests(replaced), digests(original));
    drop(temporary);
    assert_eq!(
        handles(window),
        original,
        "dropping the owner did not restore borrowed icons"
    );
    assert_released(replaced);
}
