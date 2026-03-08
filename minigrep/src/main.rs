use std::env;
use std::fs;
use std::process;
use std::error::Error;
use std::path::{Path, PathBuf};

use minigrep::{search, search_case_insensitive};

fn main() {
    let config = Config::build(env::args()).unwrap_or_else(|err| {
        eprintln!("Problem parsing arguments: {err}");
        process::exit(1);
    });

    if let Err(e) = run(config) {
        eprintln!("Error occurred while running the program: {e}");
        process::exit(1);
    }
}

pub struct Config {
    pub query: String,
    pub file_path: String,
    pub ignore_case: bool,
    pub base_path: String,
    pub search_for_file: bool,
}

impl Config {
    fn build(mut args: impl Iterator<Item = String>) -> Result<Config, &'static str> {
        args.next(); // skip program name

        let query = match args.next() {
            Some(arg) => arg,
            None => return Err("Didn't get a query string"),
        };
        let file_path = match args.next() {
            Some(arg) => arg,
            None => return Err("Didn't get a file path"),
        };

        let ignore_case = env::var("IGNORE_CASE").map_or(false, |v| v == "1");

        let base_path = env::var("BASE_PATH").unwrap_or_else(|_| "C:\\".to_string());

        let search_for_file = match args.next() {
            Some(arg) => arg == "--search-for-file",
            None => false,
        };

        Ok(Config { query, file_path, ignore_case, base_path, search_for_file })
    }
}

fn run(config: Config) -> Result<(), Box<dyn Error>> {
    let file_path = if config.search_for_file {
        let file_name = Path::new(&config.file_path)
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or("Invalid file name in file_path")?;

        let base_path = Path::new(&config.base_path);

        match find_file_accessible(base_path, file_name)? {
            Some(found_path) => found_path.to_string_lossy().into_owned(),
            None => return Err("File not found".into()),
        }
    } else {
        config.file_path
    };

    let contents = fs::read_to_string(&file_path)?;

    let results = if config.ignore_case {
        search_case_insensitive(&config.query, &contents)
    } else {
        search(&config.query, &contents)
    };

    for line in results {
        println!("{line}");
    }

    Ok(())
}


fn should_skip(name: &str) -> bool {
    name.starts_with(".") ||
    name.contains("App") ||
    name.contains("Program") ||
    name.contains("Default")
}


fn find_file_accessible(base_path: &Path, file_name: &str) -> Result<Option<PathBuf>, Box<dyn Error>> {    
    // skip inaccessible directories
    let entries = match fs::read_dir(base_path) {
        Ok(entries) => entries,
        Err(_) => {
            println!("Skipping inaccessible directory: {}", base_path.display());
            return Ok(None);
        }
    };

    let mut dirs = Vec::new();
    let mut files = Vec::new();

    // Collect files and dirs separately
    for entry in entries {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => {
                println!("Skipping unreadable entry in: {}", base_path.display());
                continue;
            }
        };
        let path = entry.path();

        if let Some(file_name_str) = path.file_name().and_then(|n| n.to_str()) {
            if should_skip(file_name_str) {
                continue;
            }
        }
        if path.is_file() {
            files.push(path);
        } else if path.is_dir() {
            dirs.push(path);
        }
    }

    // Sort dirs so "Users" comes first (case-insensitive)
    if base_path.to_string_lossy().eq_ignore_ascii_case("C:\\") {
        dirs.sort_by(|a, b| {
            let a_is_users = a.file_name().and_then(|n| n.to_str()).map_or(false, |s| s.eq_ignore_ascii_case("Users"));
            let b_is_users = b.file_name().and_then(|n| n.to_str()).map_or(false, |s| s.eq_ignore_ascii_case("Users"));
            b_is_users.cmp(&a_is_users)
        });
    }

    for path in files {
        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
            if name.eq_ignore_ascii_case(file_name) {
                println!("Found file: {}", path.display());
                return Ok(Some(path));
            }
        }
    }

    for path in dirs {
        if let Some(found) = find_file_accessible(&path, file_name)? {
            return Ok(Some(found));
        }
    }

    Ok(None)
}