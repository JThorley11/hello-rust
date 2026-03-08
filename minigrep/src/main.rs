use std::env;
use std::fs;
use std::process;
use std::error::Error;
use std::path::Path;

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
    fn build(
        mut args: impl Iterator<Item = String>
    ) -> Result<Config, &'static str> {
        args.next();

        let query = match args.next() {
            Some(arg) => arg,
            None => return Err("Didn't get a query string"),
        };
        let file_path = match args.next() {
            Some(arg) => arg,
            None => return Err("Didn't get a file path"),
        };

        let ignore_case = env::var("IGNORE_CASE").map_or(false, |v| v == "1");

        let base_path = env::var("BASE_PATH").unwrap();

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
        
        let output = process::Command::new("powershell")
            .arg("-Command")
            .arg(format!("Get-ChildItem '{}' -Recurse -Filter '{}' | Select-Object -First 1 -ExpandProperty FullName", config.base_path, file_name))
            .output()?;
        
        if !output.status.success() {
            return Err("PowerShell command failed".into());
        }
        
        let found_path = String::from_utf8(output.stdout)?
            .trim()
            .to_string();
        
        if found_path.is_empty() {
            return Err("File not found".into());
        }
        
        found_path
    } else {
        config.file_path
    };

    let contents = fs::read_to_string(file_path)?;

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