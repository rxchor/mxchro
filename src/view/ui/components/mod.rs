//! Controlled, reusable UI elements. Components receive data and callbacks;
//! they do not know about MxchroApp, backend commands or persistence.

use gpui_kit::{App, Window};
use std::rc::Rc;

pub type EventHandler<E> = Rc<dyn Fn(&E, &mut Window, &mut App)>;

pub mod app_check_box;
pub mod app_confirm_dialog;
pub mod app_input_field;
pub mod app_notice;
pub mod app_select;
pub mod collection_name_popover;
pub mod collection_sidebar;
pub mod collection_toolbar;
pub mod key_select;
pub mod macro_card;
pub mod pending_style;
pub mod toggle_hotkey;

pub use app_check_box::AppCheckBox;
pub use app_confirm_dialog::{AppConfirmDialog, ConfirmDialogEvent};
pub use app_input_field::AppInputField;
pub use app_notice::{AppNotice, NoticeKind};
pub use app_select::AppSelect;
pub mod field_error;
pub mod hotkey_warnings;
