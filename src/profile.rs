use std::{fmt::Display, path::PathBuf};

#[derive(Debug, Clone)]
pub struct Profile {
    pub path: PathBuf,
}

impl Display for Profile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.path.display())
    }
}

impl From<PathBuf> for Profile {
    fn from(path: PathBuf) -> Self {
        Self { path }
    }
}
