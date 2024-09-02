mod generator;

use std::env;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

use openpgp::parse::Parse;
use openpgp::Cert;
use sequoia_openpgp as openpgp;

fn print_usage() {
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
        env!("CARGO_PKG_NAME")
    );
}

fn _load_pub_key(file_path: &str) -> Result<Cert, std::io::Error> {
    let mut file = File::open(file_path)?;
    let mut key_data = Vec::new();

    file.read_to_end(&mut key_data)?;

    match Cert::from_bytes(&key_data) {
        Ok(cert) => Ok(cert),
        Err(err) => Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            err.to_string(),
        )),
    }
}

fn _init_store(_path: &PathBuf) -> Result<(), std::io::Error> {
    std::fs::create_dir_all(_path)?;
    Ok(())
}

fn _get_entries(_path: &PathBuf) -> Result<Vec<String>, std::io::Error> {
    let mut entries = Vec::new();

    for entry in std::fs::read_dir(_path)? {
        let entry = entry?;
        if entry.path().is_file() {
            if let Some(name) = entry.path().file_stem() {
                entries.push(name.to_string_lossy().into_owned());
            }
        }
    }

    Ok(entries)
}

fn _add_entry(name: &str, cert: &Cert, path: &PathBuf) -> Result<(), std::io::Error> {
    let mut path = path.clone();
    path.push(name);
    path.set_extension("gpg");

    let mut file = File::create(&path)?;

    let data = "pass";

    Ok(())
}

fn _get_entry(_name: &str) {}

fn _update_entry(_name: &str) {}

fn _delete_entry(_name: &str) {}

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
                "-h" | "--help" => print_usage(),
                _ => {
                    eprintln!("Unknown command or invalid number of arguments. Use -h or --help to print usage.");
                }
            }
        }
        3 => {
            let command = args[1].as_str();
            match command {
                _ => {
                    eprintln!("Unknown command. Use -h or --help to print usage.");
                }
            }
        }
        _ => {
            eprintln!("Invalid number of arguments. Use -h or --help to print usage.");
        }
    }

    Ok(())
}
