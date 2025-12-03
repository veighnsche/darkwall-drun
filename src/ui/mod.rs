//! UI module - handles all TUI rendering
//!
//! Structure:
//! - `draw/` - Drawing functions split by mode (launcher, execution)
//! - `theme.rs` - Color themes and presets
//! - `layout.rs` - Grid layout logic
//! - `entry_card.rs` - Entry card widget

mod draw;
pub mod entry_card;
pub mod layout;
pub mod theme;

// Re-export main draw function (used by main.rs)
pub use draw::draw;
