mod generator;

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

// Is there smth like Doxygen for rust?
fn init_store(store_path: &PathBuf, id: &str) -> Result<(), std::io::Error> {
    // Create the directory if it doesn't exist
    std::fs::create_dir_all(&store_path)?;

    // Create the full path for the .gpg-id file
    let mut file_path = store_path.clone();
    file_path.push(".gpg-id");

    // Write the id to the .gpg-id file
    std::fs::write(&file_path, id)?;

    Ok(())
}

fn list_entries(store_path: &PathBuf) -> Result<(), std::io::Error> {
    // Verify existance of a store in the current directory
    if !store_path.exists() || !store_path.is_dir() {
        eprintln!("Couldn't find a store in the current directory.");
        std::process::exit(1);
    }

    // Loop through the files
    for entry in std::fs::read_dir(store_path)? {
        let entry = entry?; // I feel like this is a quick and dirty workaround

        if entry.path().is_file() {
            if let Some(name) = entry.path().file_stem() {
                // This feels kinda wrong
                if name != ".gpg-id" {
                    println!("{}", name.to_string_lossy());
                }
            }
        }
    }

    Ok(())
}

fn _add_entry(name: &str, store_path: &PathBuf) -> Result<(), std::io::Error> {
    let mut path = store_path.clone();
    path.push(name);
    path.set_extension("gpg");

    Ok(())
}

fn get_entry(store_path: &PathBuf, name: &str) -> Result<(), std::io::Error> {
    if !store_path.exists() || !store_path.is_dir() {
        eprintln!("Couldn't find a store in the current directory.");
        std::process::exit(1);
    }

    let mut found_entry: bool = false;

    // Loop through the files
    for entry in std::fs::read_dir(store_path)? {
        let entry = entry?; // I feel like this is a quick and dirty workaround

        if entry.path().is_file() {
            if let Some(current_name) = entry.path().file_stem() {
                if current_name == name {
                    println!("{}", current_name.to_string_lossy());
                    found_entry = true;
                    break;
                }
            }
        }
    }

    if found_entry == false {
        eprintln!("Couldn't find {} in the store.", name);
    }

    Ok(())
}

fn _update_entry(_name: &str) {}

fn delete_entry(store_path: &PathBuf, name: &str) -> Result<(), std::io::Error> {
    let mut path = store_path.clone();
    path.push(name);
    path.set_extension("gpg");

    std::fs::remove_file(path)?;

    Ok(())
}

fn main() -> Result<(), std::io::Error> {
    let args: Vec<String> = std::env::args().collect();

    let curr_dir: PathBuf = match std::env::current_dir() {
        Ok(dir) => dir,
        Err(_) => {
            eprintln!("Failed to get current directory.");
            std::process::exit(1);
        }
    };

    let mut store_path = curr_dir.clone();
    store_path.push(".my-password-store");

    match args.len() {
        1 => match list_entries(&store_path) {
            Ok(()) => {}
            Err(_) => {
                eprintln!("Couldn't access the store. Check permisions");
            }
        },
        2 => {
            let command = args[1].as_str();
            match command {
                "-h" | "--help" => print_usage(&args[0]),
                "list" => list_entries(&store_path)?,
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
                "init" => {
                    let _ = init_store(&store_path, &args[2]);
                }
                "delete" => {
                    let _ = delete_entry(&store_path, &args[2]);
                }
                "get" => {
                    let _ = get_entry(&store_path, &args[2]);
                }
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
