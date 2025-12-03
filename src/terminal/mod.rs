//! Terminal emulation using termwiz
//!
//! This module provides a proper terminal emulator that handles:
//! - ANSI escape sequences
//! - Cursor positioning
//! - Colors and attributes
//! - Screen buffer management
//!
//! # Module Structure
//!
//! - `config` - Terminal configuration and cursor types
//! - `emulator` - Core terminal emulator implementation
//! - `widget` - Ratatui widget for rendering terminal content
//! - `input` - Crossterm key conversion utilities

mod config;
mod emulator;
mod input;
mod widget;

#[cfg(test)]
mod tests;

// Re-export public API
pub use config::{CursorPosition, TerminalConfig};
pub use emulator::EmbeddedTerminal;
pub use input::{convert_keycode, convert_modifiers};
pub use widget::TerminalWidget;
