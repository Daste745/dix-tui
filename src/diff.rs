use std::{fmt, fs, path::Path};

use color_eyre::Result;
use yansi::Paint;

/// Renders a diff report between two Nix profile paths to the provided writer.
///
/// https://github.com/manic-systems/dix/blob/v2.2.0/crates/dix/src/main.rs
pub fn display_diff(writer: &mut impl fmt::Write, old_path: &Path, new_path: &Path) -> Result<()> {
    writeln!(
        writer,
        "{arrows} {old}",
        arrows = "<<<".bold(),
        old = fs::canonicalize(old_path)
            .unwrap_or_else(|_| old_path.to_path_buf())
            .display(),
    )?;
    writeln!(
        writer,
        "{arrows} {new}",
        arrows = ">>>".bold(),
        new = fs::canonicalize(new_path)
            .unwrap_or_else(|_| new_path.to_path_buf())
            .display(),
    )?;

    let report = dix::query_diff_report(old_path, new_path, false)?;
    dix::write_diff_report(writer, &report)?;

    Ok(())
}
