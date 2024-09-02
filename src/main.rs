mod generator;

use std::env;
use std::fs::File;
use std::path::PathBuf;

fn print_usage(program: &str) {
    print!(
        "Usage:\n\
            \t./{} [command]\n\
            Commands:\n\
            \tinit [gpg_id]             Add the GPG id\n\
            \tlist                      List all entries in the store\n\
            \tadd | insert [pass-name]  Add a new entry\n\
            \tget [pass-name]           Prints password\n\
            \tupdate [pass-name]        Update a password\n\
            \tdelete [pass-name]        Delete a password\n\
            Flags:\n\
            \t-h, --help                Guess.. Go ahead, guess\n",
        program
    );
}

fn _init_store(_path: &PathBuf) -> Result<(), std::io::Error> {
    std::fs::create_dir_all(_path)?;
    Ok(())
}

fn _get_entries(path: &PathBuf) -> Result<Vec<String>, std::io::Error> {
    let mut entries = Vec::new();

    for entry in std::fs::read_dir(path)? {
        let entry = entry?;
        if entry.path().is_file() {
            if let Some(name) = entry.path().file_stem() {
                entries.push(name.to_string_lossy().into_owned());
            }
        }
    }

    Ok(entries)
}

fn _add_entry(name: &str, path: &PathBuf) -> Result<(), std::io::Error> {
    let mut path = path.clone();
    path.push(name);
    path.set_extension("gpg");

    let mut file = File::create(&path)?;

    Ok(())
}

fn _get_entry(_name: &str) {}

fn _update_entry(_name: &str) {}

fn _delete_entry(path: &PathBuf, name: &str) -> Result<(), std::io::Error> {
    let mut path = path.clone();
    path.push(name);
    path.set_extension("gpg");

    std::fs::remove_file(path)?;

    Ok(())
}

fn main() -> Result<(), std::io::Error> {
    let args: Vec<String> = std::env::args().collect();

    let _curr_dir: PathBuf = match env::current_dir() {
        Ok(dir) => dir,
        Err(_) => {
            eprintln!("Failed to get current directory.");
            std::process::exit(1);
        }
    };

    match args.len() {
        1 => {}
        2 => {
            let command = args[1].as_str();
            match command {
                "-h" | "--help" => print_usage(&args[0]),
                _ => {
                    eprintln!(
                        "Unknown command or invalid number of arguments.\
                        Use -h or --help."
                    );
                }
            }
        }
        3 => {
            let command = args[1].as_str();
            match command {
                _ => {
                    eprintln!("Unknown command. Use -h or --help.");
                }
            }
        }
        _ => {
            eprintln!("Invalid number of arguments. Use -h or --help.");
        }
    }

    Ok(())
}
