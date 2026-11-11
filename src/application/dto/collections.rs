#[derive(Debug, Clone)]
pub struct CollectionDTO {
    pub id: String,
    pub name: String,
    pub enabled: bool,
    pub macros_count: usize,
    pub order: i32,
    pub toggle_enabled_hotkey: Option<String>,
}
