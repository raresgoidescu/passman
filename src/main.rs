mod generator;

use generator::generate_password;
use std::{io::Write, path::PathBuf, process::Command};

fn print_usage(program: &str) {
    println!(
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
            \t-h, --help                Guess.. Go ahead, guess",
        program
    );
}

// Is there smth like Doxygen for rust?
fn init_store(store_path: &PathBuf, id: &str) -> Result<(), std::io::Error> {
    // Create the directory if it doesn't exist
    std::fs::create_dir_all(&store_path)?;

    // Setting *sane* permissions can't be done while cross-platform
    // As long as other users can't write in the directory, I guess it's fine
    // NOTE: Feel free to contribute

    // Create the full path for the .gpg-id file
    let file_path = store_path.join(".gpg-id");

    // Write the id to the .gpg-id file
    std::fs::write(&file_path, id)?;

    Ok(())
}

fn list_entries(store_path: &PathBuf) -> Result<(), std::io::Error> {
    // Verify existance of a store in the current directory
    if !store_path.exists() || !store_path.is_dir() {
        eprintln!("Couldn't find a store in the home directory.");
        std::process::exit(1);
    }

    // TODO: Add recursivity and tree-like display

    // Loop through the files
    let entries = std::fs::read_dir(store_path)?;
    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        if !path.is_file() || path.extension().is_none() || path.extension().unwrap() != "gpg" {
            //println!("skipped : {}", path.display());
            continue;
        }
        let name = path.file_name().unwrap().to_string_lossy();
        println!("{}", name);
    }

    Ok(())
}

fn get_input(prompt: &str) -> Result<String, std::io::Error> {
    let mut input = String::new();
    print!("{}", prompt);
    std::io::stdout().flush()?;
    std::io::stdin().read_line(&mut input)?;
    Ok(input.trim().to_string())
}

fn add_entry(store_path: &PathBuf, name: &str) -> Result<(), std::io::Error> {
    // Get the recipient
    let gpg_id_file = store_path.join(".gpg-id");
    let gpg_id = std::fs::read_to_string(&gpg_id_file)?.trim().to_string();

    // Target file
    let mut output_file = store_path.join(name);
    output_file.set_extension("gpg");

    if output_file.exists() {
        eprintln!("Pass already exists.");
        return Err(std::io::Error::new(
            std::io::ErrorKind::AlreadyExists,
            "Pass already exists",
        ));
    }

    let random = get_input("Generate a random password? [y/n] ")?
        .trim()
        .to_lowercase()
        == "y";

    let password = if !random {
        loop {
            print!("Enter the password: ");
            std::io::stdout().flush()?;
            let password = rpassword::read_password()?;
            print!("Re-enter the password: ");
            std::io::stdout().flush()?;
            let password_again = rpassword::read_password()?;
            if password != password_again {
                eprintln!("Passwords don't match.");
            } else {
                break password_again;
            }
        }
    } else {
        let len: usize = get_input("Choose a length: ")?.parse().unwrap_or(25);
        let special = get_input("Include special characters? [y/n] ")?
            .trim()
            .to_lowercase()
            == "y";

        generate_password(len, special)
    };

    // Spawn a GPG process that encrypts the stdin and prints in the target file
    let mut gpg_child_ps = Command::new("gpg")
        .arg("--encrypt")
        .arg("--armor")
        .arg("--recipient")
        .arg(gpg_id)
        .arg("--output")
        .arg(output_file)
        .stdin(std::process::Stdio::piped()) // equiv: ./passucks | gpg ...
        .spawn()?;

    // GPG process is created now it waits for input
    // So we take ownership of the process and pipe in the password
    if let Some(mut stdin) = gpg_child_ps.stdin.take() {
        stdin.write_all(password.as_bytes())?;
    }

    drop(password);

    // We wait (sync) till GPG outputs everything
    let out = gpg_child_ps.wait_with_output()?;

    if out.status.success() {
        println!("Pass stored!");
        return Ok(());
    } else {
        return Err(std::io::Error::new(std::io::ErrorKind::Other, "GPG Failed"));
    }
}

fn get_entry(store_path: &PathBuf, name: &str) -> Result<(), std::io::Error> {
    let mut target_entry = store_path.join(name);
    target_entry.set_extension("gpg");

    // Check existance of target file
    if !target_entry.exists() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Entry doesn't exist",
        ));
    }

    // Use GPG to decrypt the file
    let gpg_child_out = Command::new("gpg")
        .arg("--decrypt")
        .arg(target_entry)
        .output()?;

    // Check success
    if gpg_child_out.status.success() {
        println!("{}", String::from_utf8_lossy(&gpg_child_out.stdout));
        return Ok(());
    } else {
        return Err(std::io::Error::new(std::io::ErrorKind::Other, "GPG failed"));
    }
}

fn _update_entry(_name: &str) {}

fn delete_entry(store_path: &PathBuf, name: &str) -> Result<(), std::io::Error> {
    let mut path = store_path.join(name);
    path.set_extension("gpg");

    std::fs::remove_file(path)?;

    Ok(())
}

fn main() -> Result<(), std::io::Error> {
    let args: Vec<String> = std::env::args().collect();

    let curr_dir: PathBuf = match dirs::home_dir() {
        Some(dir) => dir,
        None => {
            eprintln!("No $HOME directory set. WTF!");
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
                    if !store_path.exists() {
                        eprintln!("There is no store. Use -h or --help");
                    } else {
                        // don't know what to do with this
                        let _res = match get_entry(&store_path, &args[2]) {
                            Ok(()) => "gg".to_string(),
                            Err(err) => err.to_string(),
                        };
                    }
                }
                "add" => {
                    if !store_path.exists() {
                        eprintln!("There is no store. Use -h or --help");
                    } else {
                        let _ = add_entry(&store_path, &args[2]);
                    }
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
