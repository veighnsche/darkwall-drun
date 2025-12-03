//! UI module - handles all TUI rendering
//!
//! Structure:
//! - `draw.rs` - Main draw functions
//! - `theme.rs` - Color themes and presets
//! - `layout.rs` - Grid layout logic
//! - `entry_card.rs` - Entry card widget

mod draw;
pub mod entry_card;
pub mod layout;
pub mod theme;

// Re-export main draw function (used by main.rs)
pub use draw::draw;
