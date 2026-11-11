/// One short-lived capture, independent from macro execution and persistence.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CaptureSession(pub u32);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CaptureError {
    Unavailable,
    Busy,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CaptureOutcome {
    Key { vk: u16, key: Option<String> },
    Cancelled,
}

/// Nonblocking UI-facing capture contract. Implementations must not install hooks
/// in `start`; bootstrap owns their installation and window binding.
/// The source intercepts captured physical input before macro execution. Polling
/// resolves its catalog name after release; cancellation is scoped to its token.
pub trait KeyCaptureBackend: Send + Sync {
    fn start(&self) -> Result<CaptureSession, CaptureError>;
    fn poll(&self, session: CaptureSession) -> Option<CaptureOutcome>;
    fn cancel(&self, session: CaptureSession);
}
