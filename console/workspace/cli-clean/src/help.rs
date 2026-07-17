//! Help pages.
//!
//! The exact column widths, wrapping, and padding of each help page are
//! compatibility output taken verbatim from the binary; they are stored as
//! observed-output fixtures and emitted as opaque strings. `render_help` selects
//! the fixture for a mode.

use crate::modes::Mode;

const HELP_GLOBAL: &str = include_str!("../fixtures/help_global.txt");
const HELP_INTERACTIVE: &str = include_str!("../fixtures/help_interactive.txt");
const HELP_VARIANTS: &str = include_str!("../fixtures/help_variants.txt");
const HELP_TEST: &str = include_str!("../fixtures/help_test.txt");

/// The help page for a mode, byte-for-byte as the binary prints it.
pub fn render_help(mode: Mode) -> &'static str {
    match mode {
        Mode::Batch => HELP_GLOBAL,
        Mode::Interactive => HELP_INTERACTIVE,
        Mode::Variants => HELP_VARIANTS,
        Mode::Test => HELP_TEST,
    }
}
