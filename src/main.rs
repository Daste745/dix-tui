use std::{
    fmt::{self, Write},
    fs, io,
    path::Path,
};

use color_eyre::Result;
use yansi::Paint;

struct WriteFmt<W: io::Write>(W);

impl<W: io::Write> fmt::Write for WriteFmt<W> {
    fn write_str(&mut self, string: &str) -> fmt::Result {
        self.0.write_all(string.as_bytes()).map_err(|_| fmt::Error)
    }
}

fn main() -> Result<()> {
    color_eyre::install()?;

    // TODO)) Remove hardcoded paths
    let old_path = Path::new("/nix/var/nix/profiles/system-197-link");
    let new_path = Path::new("/nix/var/nix/profiles/system-199-link");

    display_diff(old_path, new_path)?;

    Ok(())
}

/// https://github.com/manic-systems/dix/blob/v2.2.0/crates/dix/src/main.rs
fn display_diff(old_path: &Path, new_path: &Path) -> Result<()> {
    let mut out = WriteFmt(io::stdout());

    writeln!(
        out,
        "{arrows} {old}",
        arrows = "<<<".bold(),
        old = fs::canonicalize(old_path)
            .unwrap_or_else(|_| old_path.to_path_buf())
            .display(),
    )?;
    writeln!(
        out,
        "{arrows} {new}",
        arrows = ">>>".bold(),
        new = fs::canonicalize(new_path)
            .unwrap_or_else(|_| new_path.to_path_buf())
            .display(),
    )?;

    let report = dix::query_diff_report(old_path, new_path, false)?;
    dix::write_diff_report(&mut out, &report)?;

    Ok(())
}
