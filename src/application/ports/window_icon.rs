#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WindowIconError {
    InvalidSvg,
    Native,
}

/// Bound to one application-owned window by bootstrap. No native handles or
/// platform/framework dependencies cross this contract.
pub trait WindowIconBackend {
    fn set_icon(&mut self, svg: &[u8]) -> Result<(), WindowIconError>;
}
