use std::path::Path;

use color_eyre::Result;

use crate::profile::Profile;

mod diff;
mod profile;
mod tui;

fn main() -> Result<()> {
    color_eyre::install()?;

    // TODO)) Read all profiles from `/nix/var/nix/profiles`
    let profiles: Vec<Profile> = [
        // aquilo
        // Path::new("/nix/var/nix/profiles/system-191-link"),
        // Path::new("/nix/var/nix/profiles/system-192-link"),
        // Path::new("/nix/var/nix/profiles/system-193-link"),
        // Path::new("/nix/var/nix/profiles/system-194-link"),
        // Path::new("/nix/var/nix/profiles/system-195-link"),
        // Path::new("/nix/var/nix/profiles/system-196-link"),
        // Path::new("/nix/var/nix/profiles/system-197-link"),
        // Path::new("/nix/var/nix/profiles/system-198-link"),
        // Path::new("/nix/var/nix/profiles/system-199-link"),

        // nauvis
        Path::new("/nix/var/nix/profiles/system-326-link"),
        Path::new("/nix/var/nix/profiles/system-327-link"),
        Path::new("/nix/var/nix/profiles/system-328-link"),
        Path::new("/nix/var/nix/profiles/system-329-link"),
    ]
    .iter()
    .flat_map(|p| Profile::try_from(p.to_path_buf()))
    .collect();

    let terminal = ratatui::init();
    let mut state = tui::TuiState::new(profiles);
    let result = tui::run(terminal, &mut state);
    ratatui::restore();
    result
}
