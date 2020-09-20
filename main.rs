#![deny(clippy::pedantic, clippy::nursery)]

use std::{env, error, fmt, fs, io, path, process};
use std::path::{Path, PathBuf};

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

    fn get_absolute_path<P: AsRef<Path>>(relative_path: P) -> YeetResult<PathBuf> {
        let mut absolute_path = env::current_dir()?;

        for component in relative_path.as_ref().iter() {
            if component == ".." {
                absolute_path.pop();
            } else if component != "." {
                absolute_path.push(component);
            }
        }

        Ok(absolute_path)
    }

    pub fn empty(&self) -> YeetResult<()> {
        for entry in fs::read_dir(&self.location)? {
            let path = entry?.path();

            if path.is_dir() {
                if let Err(error) = fs::remove_dir_all(&path) {
                    let name = path.to_str().unwrap_or("[could not get directory name]");
                    eprintln!("error deleting directory `{}`: {}", name, error);
                }
            } else if let Err(error) = fs::remove_file(&path) {
                let name = path.to_str().unwrap_or("[could not get file name]");
                eprintln!("error deleting file `{}`: {}", name, error);
            }

        }
        
        Ok(())
    }

    pub fn restore<P: AsRef<Path>>(&self, old_path: P) -> YeetResult<()>
    {
        let absolute_path = Self::get_absolute_path(&old_path)?;
        let home_directory = dirs::home_dir().ok_or("unable to get home directory")?;

        if absolute_path.starts_with(&self.location) {
            let path_suffix = absolute_path.strip_prefix(&self.location)?;
            let new_path = home_directory.join(path_suffix);

            fs::rename(old_path, new_path).map_err(YeetError::from)
        } else {
            Err(YeetError::Base("cannot restore a file that is not in the dumpster"))
        }
    }

    pub fn yeet<P>(&self, old_path: P) -> YeetResult<()>
        where P: AsRef<Path>
    {
        let absolute_path = Self::get_absolute_path(&old_path)?;
        let home_directory = dirs::home_dir().ok_or("unable to get home directory")?;

        if absolute_path.starts_with(&self.location) {
            Err(YeetError::Base("cannot yeet file that is already in the dumpster"))
        } else if absolute_path.starts_with(&home_directory) {
            let path_suffix = absolute_path.strip_prefix(&home_directory)?;

            let mut new_path = self.location.join(path_suffix);
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

    match args.peek().map(String::as_str) {
        Some("--restore") => {
            args.next();

            for filename in args {
                if let Err(error) = dumpster.restore(&filename) {
                    eprintln!("{}: {}", filename, error);
                }
            }
        },
        Some("--empty") => {
            if let Err(error) = dumpster.empty() {
                eprintln!("{}", error);
            }
        },
        _ => for filename in args {
            if let Err(error) = dumpster.yeet(&filename) {
                eprintln!("{}: {}", filename, error);
            }
        }
    };
}
