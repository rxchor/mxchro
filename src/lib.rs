//! Native Mxchro application: framework-independent domain, Windows input and
//! a GPUI-KIT frontend connected by ordered Rust commands.

pub mod application;
pub mod bootstrap;
pub mod domain;
pub mod infrastructure;
pub mod view;

rust_i18n::i18n!("locales", fallback = "en");
