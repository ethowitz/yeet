#![deny(clippy::pedantic, clippy::nursery)]

use std::{env, error, fmt, fs, io, process};
use std::path::PathBuf;

#[derive(Debug)]
enum YeetError {
    Base(&'static str),
    Io(io::Error),
}

impl From<io::Error> for YeetError {
    fn from(err: io::Error) -> YeetError {
        YeetError::Io(err)
    }
}

impl From<&'static str> for YeetError {
    fn from(err: &'static str) -> YeetError {
        YeetError::Base(err)
    }
}

impl fmt::Display for YeetError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            YeetError::Base(ref err) => write!(f, "{}", err),
            YeetError::Io(ref err) => write!(f, "{}", err),
        }
    }
}

impl error::Error for YeetError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match *self {
            YeetError::Io(ref err) => err.source(),
            _ => None,
        }
    }
}

type YeetResult<T> = Result<T, YeetError>;

#[derive(Debug)]
struct Dumpster {
    location: PathBuf,
}

impl Dumpster {
    const DEFAULT_DUMPSTER_NAME: &'static str = ".dumpster";
    const MAX_NUMBER_OF_DUPLICATES: u16 = u16::MAX;

    fn create_dumpster(location: &PathBuf) -> YeetResult<()> {
        fs::create_dir(location).map_err(YeetError::from)
    }

    fn default_dumpster_location() -> YeetResult<PathBuf> {
        dirs::home_dir().ok_or(YeetError::Base("failed to get home directory"))
    }

    fn dumpster_location() -> YeetResult<PathBuf> {
        let mut path = Self::default_dumpster_location()?;
        path.push(PathBuf::from(Self::DEFAULT_DUMPSTER_NAME));

        Ok(path)
    }

    pub fn with_default_location() -> YeetResult<Self> {
        let location: PathBuf = Self::dumpster_location()?;

        if !location.exists() {
            Self::create_dumpster(&location)?;
        }

        Ok(Self { location })
    }

    fn get_filename<'a>(path: &'a PathBuf) -> YeetResult<&'a str> {
        path.file_name().and_then(|s| s.to_str()).ok_or(YeetError::Base("could not get filename"))
    }

    fn generate_filename(&self, original_filename: &str, new_path: &PathBuf) -> YeetResult<String> {
        let is_available = |filename: &str| -> bool { !new_path.join(filename).exists() };

        if is_available(original_filename) {
            Ok(String::from(original_filename))
        } else {
            (0..Self::MAX_NUMBER_OF_DUPLICATES).
                map(|n| format!("{}.{}", original_filename, n)).
                // I guess deref coercion does not happen if I just pass `filename_available`?
                find(|filename| is_available(filename)).
                ok_or(YeetError::Base("max number of duplicate files reached"))
        }
    }

    fn get_absolute_path(relative_path: &PathBuf) -> YeetResult<PathBuf> {
        let mut absolute_path = env::current_dir()?;

        for component in relative_path.iter() {
            if component == ".." {
                absolute_path.pop();
            } else if component != "." {
                absolute_path.push(component);
            }
        }

        Ok(absolute_path)
    }

    fn generate_path(&self, old_path: &PathBuf) -> YeetResult<PathBuf> {
        let absolute_path = Self::get_absolute_path(old_path)?;
        let home_directory = dirs::home_dir().ok_or("unable to get home directory")?;

        if absolute_path.starts_with(&home_directory) {
            let path_suffix = absolute_path.strip_prefix(&home_directory).
                map_err(|_| "could not generate new path")?;

            let mut new_path_prefix = home_directory;
            new_path_prefix.push(Self::DEFAULT_DUMPSTER_NAME);

            let mut new_path = new_path_prefix.join(path_suffix);
            new_path.pop();

            let old_filename = Self::get_filename(&absolute_path)?;
            let new_filename = self.generate_filename(old_filename, &new_path)?;
            fs::create_dir_all(&new_path).map_err(|_| "could not create requisite directories")?;

            new_path.push(new_filename);

            Ok(new_path)
        } else {
            Err(YeetError::Base("cannot yeet file outside of home directory"))
        }
    }

    pub fn yeet_file(&self, old_path: PathBuf) -> YeetResult<()> {
        let new_path = self.generate_path(&old_path)?;

        // TODO: handle permissions errors (return OS error or check for permissions errors)
        fs::rename(old_path, new_path).or(Err("failed to move file to dumpster"))?;
        Ok(())
    }
}

fn main() {
    let dumpster = Dumpster::with_default_location().unwrap_or_else(|error| {
        eprintln!("failed to initialize dumpster: {}", error);
        process::exit(1);
    });

    for filename in env::args().skip(1) {
        let path = PathBuf::from(&filename);

        if let Err(error) = dumpster.yeet_file(path) {
            eprintln!("{}: {}", filename, error);
        }
    }
}
