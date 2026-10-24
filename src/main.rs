use std::fs;

use color_eyre::Result;

use crate::profile::Profile;

mod diff;
mod profile;
mod tui;

const PROFILES_DIR: &str = "/nix/var/nix/profiles";

fn main() -> Result<()> {
    color_eyre::install()?;

    let paths = fs::read_dir(PROFILES_DIR)?;
    let mut profiles: Vec<Profile> = paths
        .flatten()
        .filter(|p| p.path().is_symlink())
        .filter(|p| !(p.file_name().to_string_lossy() == "system"))
        .filter(|p| !(p.file_name().to_string_lossy() == "default"))
        .flat_map(|p| Profile::try_from(p.path()))
        .collect();
    println!("{:?}", profiles.clone());
    profiles.sort_by_key(|p| p.generation);

    let terminal = ratatui::init();
    let mut state = tui::TuiState::new(profiles);
    let result = tui::run(terminal, &mut state);
    ratatui::restore();
    result
}
