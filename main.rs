#![deny(clippy::pedantic, clippy::nursery)]

use std::{env, fs, io, path::PathBuf, process};

const DEFAULT_DUMPSTER_FILENAME: &str = ".dumpster";

fn create_dumpster() -> io::Result<()> {
    fs::create_dir(dumpster_location())
}

fn default_dumpster_location() -> PathBuf {
    dirs::home_dir().unwrap_or_else(|| {
        eprintln!("error: unable to get home directory");
        process::exit(1);
    })
}

fn dumpster_location() -> PathBuf {
    let mut path = default_dumpster_location();
    path.push(PathBuf::from(DEFAULT_DUMPSTER_FILENAME));

    path
}

fn yeet_file(filename: &str) -> io::Result<()> {
    let mut new_path = dumpster_location();
    new_path.push(filename);

    fs::rename(filename, new_path)
}

fn main() {
    if !dumpster_location().exists() {
        if let Err(error) = create_dumpster() {
            eprintln!("could not create dumpster: {}", error);
            process::exit(1);
        }
    }

    for filename in env::args().skip(1) {
        if let Err(error) = yeet_file(&filename) {
            eprintln!("failed to move file {} to the dumpster: {}", filename, error);
        }
    }
}
