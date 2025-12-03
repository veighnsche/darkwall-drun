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

// Re-export main draw function
pub use draw::draw;

// Re-export commonly used types
pub use entry_card::{EntryCard, EntryDisplayConfig};
pub use layout::GridLayout;
pub use theme::Theme;
