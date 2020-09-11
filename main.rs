#![deny(clippy::pedantic, clippy::nursery)]

use std::{env, fs, process};
use std::io::Result as IOResult;
use std::path::PathBuf;

struct Dumpster {
    location: PathBuf,
}

impl Dumpster {
    const DEFAULT_DUMPSTER_NAME: &'static str = ".dumpster";

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

    pub fn with_default_location() -> Result<Dumpster, &'static str> {
        let location: PathBuf = Self::dumpster_location()?;

        if !location.exists() {
            Self::create_dumpster(&location)?;
        }

        Ok(Dumpster { location: location })
    }

    // Instance methods
    // TODO: generate new file names for duplicates (filename.0, filename.1)
    //fn generate_filename(original_filename: &str) -> &str {

    //}

    pub fn yeet_file(&self, file: File) -> IOResult<()> {
        let mut new_path = self.location.clone();
        new_path.push(&file.location);

        fs::rename(&file.location, new_path)
    }
}

struct File {
    location: PathBuf,
}

impl File {
    fn from_string(filename: &str) -> File {
        File { location: PathBuf::from(filename) }
    }
}

fn main() {
    let dumpster = Dumpster::with_default_location().unwrap_or_else(|error| {
        eprintln!("failed to initialize dumpster: {}", error);
        process::exit(1);
    });

    for filename in env::args().skip(1) {
        let file = File::from_string(&filename);

        if let Err(error) = dumpster.yeet_file(file) {
            eprintln!(
                "failed to move file {} to the dumpster: {}",
                filename, error
            );
        }
    }
}
