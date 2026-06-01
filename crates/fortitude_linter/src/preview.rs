//! Helpers to test if a specific preview style is enabled or not.
//!
//! The motivation for these functions isn't to avoid code duplication but to
//! ease promoting preview behavior to stable. The challenge with directly
//! checking `preview` is that it is unclear which specific feature this preview
//! check is for. Having named functions simplifies the promotion: Simply delete
//! the function and let Rust tell you which checks you have to remove.

use crate::settings::PreviewMode;

pub const fn is_warning_severity_enabled(preview: PreviewMode) -> bool {
    preview.is_enabled()
}

pub const fn is_show_diff_enabled(preview: PreviewMode) -> bool {
    preview.is_enabled()
}
