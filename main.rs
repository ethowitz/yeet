#![deny(clippy::pedantic, clippy::nursery)]

use std::{env, error, fmt, fs, io, path, process};
use std::path::PathBuf;

#[derive(Debug)]
enum YeetError {
    // I will reevaluate this depending on how large this project becomes
    Base(&'static str),
    Io(io::Error),
    Prefix(path::StripPrefixError),
}

impl From<&'static str> for YeetError {
    fn from(err: &'static str) -> YeetError {
        YeetError::Base(err)
    }
}

impl From<io::Error> for YeetError {
    fn from(err: io::Error) -> YeetError {
        YeetError::Io(err)
    }
}

impl From<path::StripPrefixError> for YeetError {
    fn from(err: path::StripPrefixError) -> YeetError {
        YeetError::Prefix(err)
    }
}

impl fmt::Display for YeetError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            YeetError::Base(ref err) => write!(f, "{}", err),
            YeetError::Io(ref err) => write!(f, "{}", err),
            YeetError::Prefix(ref err) => write!(f, "{}", err),
        }
    }
}

impl error::Error for YeetError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match *self {
            YeetError::Base(_) => None,
            YeetError::Io(ref err) => err.source(),
            YeetError::Prefix(ref err) => err.source(),
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

    pub fn restore(&self, old_path: PathBuf) -> YeetResult<()> {
        let absolute_path = Self::get_absolute_path(&old_path)?;
        let home_directory = dirs::home_dir().ok_or("unable to get home directory")?;
        let dumpster_prefix = home_directory.join(Self::DEFAULT_DUMPSTER_NAME);

        if absolute_path.starts_with(&dumpster_prefix) {
            let path_suffix = absolute_path.strip_prefix(dumpster_prefix)?;
            let new_path = home_directory.join(path_suffix);

            fs::rename(old_path, new_path).map_err(YeetError::from)
        } else {
            Err(YeetError::Base("cannot restore a file that is not in the dumpster"))
        }
    }

    pub fn yeet(&self, old_path: PathBuf) -> YeetResult<()> {
        let absolute_path = Self::get_absolute_path(&old_path)?;
        let home_directory = dirs::home_dir().ok_or("unable to get home directory")?;
        let dumpster_prefix = home_directory.join(Self::DEFAULT_DUMPSTER_NAME);

        if absolute_path.starts_with(&dumpster_prefix) {
            Err(YeetError::Base("cannot yeet file that is already in the dumpster"))
        } else if absolute_path.starts_with(&home_directory) {
            let path_suffix = absolute_path.strip_prefix(&home_directory)?;

            let mut new_path = dumpster_prefix.join(path_suffix);
            new_path.pop();

            let old_filename = Self::get_filename(&absolute_path)?;
            let new_filename = self.generate_filename(old_filename, &new_path)?;
            fs::create_dir_all(&new_path)?;

            new_path.push(new_filename);

            fs::rename(old_path, new_path).map_err(YeetError::from)
        } else {
            Err(YeetError::Base("cannot yeet file that is outside of the home directory"))
        }
    }
}

fn main() {
    let dumpster = Dumpster::with_default_location().unwrap_or_else(|error| {
        eprintln!("failed to initialize dumpster: {}", error);
        process::exit(1);
    });

    let mut args = env::args().skip(1).peekable();
    let yeet_file = match args.peek().map(String::as_str) {
        Some("--restore") | Some("-r") => {
            args.next();
            false
        },
        _ => true,
    };

    for filename in args {
        let path = PathBuf::from(&filename);
        let result = if yeet_file { dumpster.yeet(path) } else { dumpster.restore(path) };

        if let Err(error) = result {
            eprintln!("{}: {}", filename, error);
        }
    }
}
