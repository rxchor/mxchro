//! Capture is intercepted before macro processing. Hook-facing state uses only
//! atomics: no channels, allocations, locks, repository calls or UI callbacks.
use crate::application::ports::{
    CaptureError, CaptureOutcome, CaptureSession, InputBackend, KeyCaptureBackend,
};
use std::sync::{
    Arc,
    atomic::{AtomicBool, AtomicIsize, AtomicU32, AtomicU64, Ordering},
};
use windows::Win32::UI::WindowsAndMessaging::GetForegroundWindow;

const WAITING: u64 = 1;
const KEY: u64 = 2;
const CANCELLED: u64 = 3;
const ESCAPE: u16 = 0x1B;

#[derive(Default)]
struct CaptureState {
    next: AtomicU32,
    state: AtomicU64,
    pressed: [AtomicU64; 4],
    swallowed: [AtomicU64; 4],
}

impl CaptureState {
    fn start(&self) -> Result<CaptureSession, CaptureError> {
        let id = self.next.fetch_add(1, Ordering::Relaxed).wrapping_add(1);
        let session = CaptureSession(id);
        self.state
            .compare_exchange(
                0,
                (u64::from(id) << 32) | WAITING,
                Ordering::AcqRel,
                Ordering::Acquire,
            )
            .map(|_| session)
            .map_err(|_| CaptureError::Busy)
    }

    fn cancel(&self, session: CaptureSession) {
        let mut state = self.state.load(Ordering::Acquire);
        while state != 0 && state >> 32 == u64::from(session.0) {
            match self
                .state
                .compare_exchange(state, 0, Ordering::AcqRel, Ordering::Acquire)
            {
                Ok(_) => break,
                Err(current) => state = current,
            }
        }
        // Do not clear swallowed releases: their corresponding downs never
        // reached the OS or engine, even if the UI cancels/closes in the meantime.
    }

    fn poll(&self, session: CaptureSession) -> Option<Result<u16, ()>> {
        let state = self.state.load(Ordering::Acquire);
        if state == 0 || state >> 32 != u64::from(session.0) {
            return None;
        }
        let phase = state & 0xFF;
        let outcome = match phase {
            CANCELLED => Err(()),
            KEY => {
                let vk = ((state >> 8) & 0xFFFF) as u16;
                // Commit only after release, never while a captured key is held.
                if self.pressed[usize::from(vk / 64)].load(Ordering::Acquire) & (1 << (vk % 64))
                    != 0
                {
                    return None;
                }
                Ok(vk)
            }
            _ => return None,
        };
        self.state
            .compare_exchange(state, 0, Ordering::AcqRel, Ordering::Acquire)
            .ok()?;
        Some(outcome)
    }

    fn process(&self, vk: u16, down: bool, up: bool, injected: bool, foreground: bool) -> bool {
        if injected || vk >= 256 || (!down && !up) {
            return false;
        }
        let index = usize::from(vk / 64);
        let bit = 1u64 << (vk % 64);
        let was_pressed = if down {
            self.pressed[index].fetch_or(bit, Ordering::AcqRel) & bit != 0
        } else {
            self.pressed[index].fetch_and(!bit, Ordering::AcqRel) & bit != 0
        };
        if self.swallowed[index].load(Ordering::Acquire) & bit != 0 {
            if up {
                self.swallowed[index].fetch_and(!bit, Ordering::AcqRel);
            }
            return true;
        }
        let state = self.state.load(Ordering::Acquire);
        if state == 0 {
            return false;
        }
        if !foreground {
            let _ = self.state.compare_exchange(
                state,
                (state & 0xFFFF_FFFF_0000_0000) | CANCELLED,
                Ordering::AcqRel,
                Ordering::Acquire,
            );
            return false;
        }
        // Releases and repeats of keys held before arming still reach the engine
        // (in particular KeyUp must be able to stop an existing Hold macro).
        if !down || was_pressed {
            return false;
        }
        if state & 0xFF == WAITING {
            let phase = if vk == ESCAPE { CANCELLED } else { KEY };
            let captured = (state & 0xFFFF_FFFF_0000_0000) | (u64::from(vk) << 8) | phase;
            if self
                .state
                .compare_exchange(state, captured, Ordering::AcqRel, Ordering::Acquire)
                .is_err()
            {
                return false;
            }
        }
        // Also swallow additional buttons/keys in this gesture, not just its
        // first key, so Shift+A cannot fire A while Shift is being captured.
        self.swallowed[index].fetch_or(bit, Ordering::AcqRel);
        true
    }
}

pub struct WindowsKeyCapture {
    state: CaptureState,
    input: Arc<dyn InputBackend>,
    window: AtomicIsize,
    available: AtomicBool,
}

impl WindowsKeyCapture {
    pub fn new(input: Arc<dyn InputBackend>) -> Self {
        Self {
            state: CaptureState::default(),
            input,
            window: AtomicIsize::new(0),
            available: AtomicBool::new(false),
        }
    }

    pub fn bind_window(&self, hwnd: isize) {
        self.window.store(hwnd, Ordering::Release);
    }

    pub(super) fn set_available(&self, available: bool) {
        self.available.store(available, Ordering::Release);
    }

    fn owns_foreground(&self) -> bool {
        let window = self.window.load(Ordering::Acquire);
        window != 0 && unsafe { GetForegroundWindow() }.0 as isize == window
    }

    pub(super) fn intercept(&self, vk: u16, down: bool, up: bool, injected: bool) -> bool {
        let foreground = self.state.state.load(Ordering::Acquire) == 0 || self.owns_foreground();
        self.state.process(vk, down, up, injected, foreground)
    }
}

impl KeyCaptureBackend for WindowsKeyCapture {
    fn start(&self) -> Result<CaptureSession, CaptureError> {
        if !self.available.load(Ordering::Acquire) || !self.owns_foreground() {
            return Err(CaptureError::Unavailable);
        }
        self.state.start()
    }

    fn poll(&self, session: CaptureSession) -> Option<CaptureOutcome> {
        self.state.poll(session).map(|result| match result {
            Ok(vk) => CaptureOutcome::Key {
                vk,
                key: self.input.vk_to_string(&vk).map(str::to_owned),
            },
            Err(()) => CaptureOutcome::Cancelled,
        })
    }

    fn cancel(&self, session: CaptureSession) {
        self.state.cancel(session);
    }
}

#[cfg(test)]
#[path = "../../../tests/unit/infrastructure/capture.rs"]
mod tests;
