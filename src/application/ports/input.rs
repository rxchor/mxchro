/// Platform input operations required by the domain. Implemented by Windows,
/// or by a fake in tests; the core never imports platform APIs.
pub trait InputBackend: Send + Sync {
    fn string_to_vk(&self, key: &str) -> Option<u16>;
    fn vk_to_string(&self, vk: &u16) -> Option<&str>;
    fn strings_vec(&self) -> Vec<String>;
    fn key_down(&self, vk: u16);
    fn key_up(&self, vk: u16);
}
