#![deny(clippy::pedantic, clippy::nursery)]

use std::{env, fs, process};
use std::path::PathBuf;

struct Dumpster {
    location: PathBuf,
}

impl Dumpster {
    const DEFAULT_DUMPSTER_NAME: &'static str = ".dumpster";
    const MAX_NUMBER_OF_DUPLICATES: u16 = u16::MAX;

    fn create_dumpster(location: &PathBuf) -> Result<(), &'static str> {
        fs::create_dir(location).or(Err("could not create dumpster"))
    }

    fn default_dumpster_location() -> Result<PathBuf, &'static str> {
        dirs::home_dir().ok_or("unable to get home directory")
    }

    fn dumpster_location() -> Result<PathBuf, &'static str> {
        let mut path = Self::default_dumpster_location()?;
        path.push(PathBuf::from(Self::DEFAULT_DUMPSTER_NAME));

        Ok(path)
    }

    pub fn with_default_location() -> Result<Self, &'static str> {
        let location: PathBuf = Self::dumpster_location()?;

        if !location.exists() {
            Self::create_dumpster(&location)?;
        }

        Ok(Self { location })
    }

    fn generate_filename(&self, original_filename: &str) -> Result<String, &'static str> {
        let filename_available = |filename: &str| -> bool {
            let mut candidate = self.location.clone();
            candidate.push(filename);
            
            !candidate.exists()
        };

        if filename_available(original_filename) {
            Ok(String::from(original_filename))
        } else {
            (0..Self::MAX_NUMBER_OF_DUPLICATES).
                map(|n| format!("{}.{}", original_filename, n)).
                // I guess deref coercion does not happen if I just pass `filename_available`?
                find(|s| filename_available(s)).
                ok_or("max number of duplicate files reached")
        }
    }

    pub fn yeet_file(&self, filename: &str) -> Result<(), &'static str> {
        let mut new_path = self.location.clone();
        let new_filename = self.generate_filename(filename)?;
        new_path.push(new_filename);

        fs::rename(filename, new_path).or(Err("failed to move file to dumpster"))?;
        Ok(())
    }
}

fn main() {
    let dumpster = Dumpster::with_default_location().unwrap_or_else(|error| {
        eprintln!("failed to initialize dumpster: {}", error);
        process::exit(1);
    });

    for filename in env::args().skip(1) {
        if let Err(error) = dumpster.yeet_file(&filename) {
            eprintln!(
                "failed to move file {} to the dumpster: {}",
                filename, error
            );
        }
    }
}
