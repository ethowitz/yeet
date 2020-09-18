#![deny(clippy::pedantic, clippy::nursery)]

use std::{env, fs, process};
use std::path::PathBuf;

// TODO: convert to
// type YeetError<T> = Result<T, impl Error>;
type YeetError<T> = Result<T, &'static str>;

struct Dumpster {
    location: PathBuf,
}

impl Dumpster {
    const DEFAULT_DUMPSTER_NAME: &'static str = ".dumpster";
    const MAX_NUMBER_OF_DUPLICATES: u16 = u16::MAX;

    fn create_dumpster(location: &PathBuf) -> YeetError<()> {
        fs::create_dir(location).or(Err("could not create dumpster"))
    }

    fn default_dumpster_location() -> YeetError<PathBuf> {
        dirs::home_dir().ok_or("unable to get home directory")
    }

    fn dumpster_location() -> YeetError<PathBuf> {
        let mut path = Self::default_dumpster_location()?;
        path.push(PathBuf::from(Self::DEFAULT_DUMPSTER_NAME));

        Ok(path)
    }

    pub fn with_default_location() -> YeetError<Self> {
        let location: PathBuf = Self::dumpster_location()?;

        if !location.exists() {
            Self::create_dumpster(&location)?;
        }

        Ok(Self { location })
    }

    fn get_filename<'a>(path: &'a PathBuf) -> YeetError<&'a str> {
        //let get_error_message = || -> &'static str {
            //match path.to_str() {
                //Some(path_as_str) =>
                  //format!("could not get filename for argument `{}'", path_as_str).as_str(),
                //None => "could not get the filename for an argument"
            //}
                //None => "could not get the filename for an argument"
        //};

        path.file_name().and_then(|s| s.to_str()).
            ok_or("could not get the filename for an argument")
    }

    fn generate_filename(&self, original_filename: &str, new_path: &PathBuf) -> YeetError<String> {
        let is_available = |filename: &str| -> bool {
            let candidate = new_path.join(filename);
            
            !candidate.exists()
        };

        if is_available(original_filename) {
            Ok(String::from(original_filename))
        } else {
            (0..Self::MAX_NUMBER_OF_DUPLICATES).
                map(|n| format!("{}.{}", original_filename, n)).
                // I guess deref coercion does not happen if I just pass `filename_available`?
                find(|filename| is_available(filename)).
                ok_or("max number of duplicate files reached")
        }
    }

    fn get_absolute_path(relative_path: &PathBuf, current_dir: &PathBuf) -> YeetError<PathBuf> {
        let mut absolute_path = current_dir.clone();

        for component in relative_path.iter() {
            if component == ".." {
                absolute_path.pop();
            } else if component != "." {
                absolute_path.push(component);
            }
        }

        Ok(absolute_path)
    }

    fn generate_path(&self, old_path: &PathBuf) -> YeetError<PathBuf> {
        let current_dir = env::current_dir().map_err(|_| "failed to get current directory")?;
        let absolute_path = Self::get_absolute_path(old_path, &current_dir)?;
        let home_directory = dirs::home_dir().ok_or("unable to get home directory")?;

        if current_dir.starts_with(&home_directory) {
            let old_filename = Self::get_filename(&absolute_path)?;
            let mut new_path_prefix = home_directory.clone();

            new_path_prefix.push(Self::DEFAULT_DUMPSTER_NAME);
            let path_suffix = absolute_path.strip_prefix(home_directory).
                map_err(|_| "could not generate new path")?;

            let mut new_path = new_path_prefix.join(path_suffix);
            new_path.pop();

            let new_filename = self.generate_filename(old_filename, &new_path)?;
            fs::create_dir_all(&new_path).map_err(|_| "could not create requisite directories")?;

            new_path.push(new_filename);

            Ok(new_path)
        } else {
            Err("cannot yeet file outside of home directory")
        }
    }

    pub fn yeet_file(&self, old_path: PathBuf) -> YeetError<()> {
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
            eprintln!(
                "failed to move file {} to the dumpster: {}",
                filename, error
            );
        }
    }
}
