mod generator;

use generator::generate_password;
use std::{io::Write, os::unix::fs::PermissionsExt, path::PathBuf, process::Command};

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
    let perm = std::fs::Permissions::from_mode(0o077);
    // Create the directory if it doesn't exist
    std::fs::create_dir_all(&store_path)?;
    std::fs::set_permissions(store_path, perm)?;

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

    println!("============== I don't feel like I need a loop ================");

    let ls_out = Command::new("ls").arg(store_path).output()?;

    if ls_out.status.success() {
        println!("{}", String::from_utf8_lossy(&ls_out.stdout));
    } else {
        println!("ayaye");
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
    let mut gpg_id_file = store_path.clone();
    gpg_id_file.push(".gpg-id");
    let gpg_id = std::fs::read_to_string(gpg_id_file)?;

    // Target file
    let mut output_file = store_path.clone();
    output_file.push(name);
    output_file.set_extension("gpg");

    if output_file.exists() {
        eprintln!("Pass already exists.");
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Pass already exists",
        ));
    }

    let mut choice = String::new();
    print!("Generate a random password? [y/n] ");
    std::io::stdout().flush()?;
    std::io::stdin().read_line(&mut choice)?;

    let random = match choice.trim().to_lowercase().as_str() {
        "y" => true,
        _ => false,
    };

    let mut password;

    if !random {
        let mut password_again;
        loop {
            print!("Enter the password: ");
            let _ = std::io::stdout().flush();
            password = rpassword::read_password()?;
            print!("Re-enter the password: ");
            let _ = std::io::stdout().flush();
            password_again = rpassword::read_password()?;
            if password != password_again {
                eprintln!("Passwords don't match.");
            } else {
                drop(password_again);
                break;
            }
        }
    } else {
        let len_input = get_input("Choose a length: ")?;
        let len: usize = len_input
            .parse()
            .map_err(|_| std::io::Error::new(std::io::ErrorKind::InvalidInput, "Invalid length"))?;

        let special_input = get_input("Do you want special characters? [Y/n] ")?;
        let special = match special_input.to_lowercase().as_str() {
            "y" => true,
            _ => false,
        };

        password = generate_password(len, special);
    }

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
    let mut target_entry = store_path.clone();
    target_entry.push(name);
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
    let mut path = store_path.clone();
    path.push(name);
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
