#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum HotkeyOwnerDTO {
    Macro(String),
    Collection(String),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum HotkeyIssueKind {
    GroupedMacro,
    Duplicate,
    InvalidKey,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct HotkeyIssueDTO {
    pub owner: HotkeyOwnerDTO,
    pub hotkey: String,
    pub kind: HotkeyIssueKind,
}
