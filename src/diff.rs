use std::{cmp::Ordering, fmt};

use color_eyre::Result;
use dix::{DiffReport, PathStats};
use size::Size;
use yansi::{Paint, Painted};

use crate::profile::Profile;

#[derive(Debug)]
pub struct Diff {
    old: Profile,
    new: Profile,
    report: DiffReport,
    /// Full output of `display_diff`
    pub formatted_report: String,
    /// `PATHS: old -> new (+x, -y)`
    pub path_stats: String,
    /// `SIZE: old -> new` and `DIFF: +x`
    pub size_diff: String,
}

impl Diff {
    pub fn new(old: Profile, new: Profile) -> Result<Self> {
        let report = dix::query_diff_report(&old.path, &new.path, false)?;

        let mut formatted_report = String::new();
        display_diff(&mut formatted_report, &old, &new, &report)?;

        let mut path_stats = String::new();
        write_path_stats(&mut path_stats, report.path_stats())?;

        let mut size_diff = String::new();
        write_size_diff(&mut size_diff, report.size_old(), report.size_new())?;

        Ok(Self {
            old,
            new,
            report,
            formatted_report,
            path_stats,
            size_diff,
        })
    }
}

/// Renders a diff report between two Nix profile paths to the provided writer.
///
/// Partially copied from `main::display_diff`:
/// https://github.com/manic-systems/dix/blob/v2.2.0/crates/dix/src/main.rs#L169
pub fn display_diff(
    writer: &mut impl fmt::Write,
    old: &Profile,
    new: &Profile,
    report: &DiffReport,
) -> fmt::Result {
    writeln!(
        writer,
        "{arrows} {old}",
        arrows = "<<<".bold().red(),
        old = old.canonical_path.display(),
    )?;
    writeln!(
        writer,
        "{arrows} {new}",
        arrows = ">>>".bold().green(),
        new = new.canonical_path.display(),
    )?;

    dix::write_diff_report(writer, report)?;

    Ok(())
}

// TODO)) Remove once exposed by `dix`
/// See `dix::render::write_path_stats`:
/// https://github.com/manic-systems/dix/blob/v2.2.0/crates/dix/src/render.rs#L172
fn write_path_stats(writer: &mut impl fmt::Write, stats: PathStats) -> fmt::Result {
    let added = format!("+{}", stats.added_count());
    let removed = format!("-{}", stats.removed_count());

    writeln!(
        writer,
        "{header}: {old} -> {new} ({added}, {removed})",
        header = "PATHS".bold(),
        old = stats.old_count().red(),
        new = stats.new_count().green(),
        added = Painted::new(added.as_str()).green(),
        removed = Painted::new(removed.as_str()).red(),
    )
}

// TODO)) Remove once exposed by `dix`
/// See `dix::render::write_size_diff`:
/// https://github.com/manic-systems/dix/blob/v2.2.0/crates/dix/src/render.rs#L148
fn write_size_diff(writer: &mut impl fmt::Write, size_old: Size, size_new: Size) -> fmt::Result {
    let size_diff = size_new - size_old;

    writeln!(
        writer,
        "{header}: {size_old} -> {size_new}",
        header = "SIZE".bold(),
        size_old = size_old.red(),
        size_new = size_new.green(),
    )?;

    let (sign, styled_diff) = match size_diff.bytes().cmp(&0) {
        Ordering::Less => ("", size_diff.red()),
        Ordering::Equal => ("", size_diff.resetting()),
        Ordering::Greater => ("+", size_diff.green()),
    };

    writeln!(writer, "{}: {sign}{styled_diff}", "DIFF".bold())
}
