use std::{fmt::Display, path::PathBuf};

#[derive(Debug, Clone)]
pub struct Profile {
    pub path: PathBuf,
    pub canonical_path: PathBuf,
    pub generation: usize,
}

impl Display for Profile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.generation)
    }
}

impl TryFrom<PathBuf> for Profile {
    type Error = std::io::Error;

    fn try_from(value: PathBuf) -> Result<Self, Self::Error> {
        // system-123-link -> 123
        let generation = value
            .display()
            .to_string()
            .split('-')
            .nth(1)
            .unwrap_or("0")
            .parse()
            .unwrap_or(0);

        let canonical_path = value.canonicalize()?;
        Ok(Self {
            path: value,
            canonical_path,
            generation,
        })
    }
}
