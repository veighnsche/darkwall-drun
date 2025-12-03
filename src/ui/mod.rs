//! UI module - handles all TUI rendering
//!
//! Structure:
//! - `draw.rs` - Main draw functions (current implementation)
//! - `theme.rs` - Color themes and presets (Unit 5.4.3) - PREPARED
//! - `layout.rs` - Grid layout logic (Unit 5.4.1) - PREPARED
//! - `entry_card.rs` - Entry card widget (Unit 5.4.2) - PREPARED
//!
//! ## For Next Team (Unit 5.4)
//!
//! The theme, layout, and entry_card modules are prepared and tested.
//! To integrate them:
//! 1. Update draw.rs to use Theme instead of hardcoded colors
//! 2. Update draw.rs to use GridLayout for 2-column layout
//! 3. Update draw.rs to use EntryCard widget for entry rendering
//! 4. Update config.rs to load theme/layout config
//! 5. Wire everything together in main.rs

mod draw;
pub mod theme;
pub mod layout;
pub mod entry_card;

// Re-export main draw function (current implementation)
pub use draw::draw;

// Re-export new components for Unit 5.4 integration
pub use theme::Theme;
pub use layout::GridLayout;
pub use entry_card::{EntryCard, EntryDisplayConfig};
