use std::process::{exit, Command};
use std::io::{stdin, stdout, Write, BufRead};
use std::path::Path;

#[link(name = "c")]
extern "C" {
    fn getuid() -> u32;
}

fn pacman_contrib_is_installed() -> bool {
    Command::new("pacman")
        .arg("-Q").arg("pacman-contrib")
        .output().unwrap().status.success()
}

fn update_with_aur_helper() {
    if Path::new("/bin/yay").exists() {
        Command::new("yay").status().unwrap();
    }
    else if Path::new("/bin/trizen").exists() {
        Command::new("trizen").status().unwrap();
    }
    else if Path::new("/bin/pikaur").exists() {
        Command::new("pikaur").status().unwrap();
    }
    else if Path::new("/bin/paru").exists() {
        Command::new("paru").status().unwrap();
    }
    else {
        eprintln!("No AUR helper found, skipping...");
    }
}

fn prune_orphans() {
    let mut orphans = Vec::new();
    Command::new("pacman")
        .arg("-Qdtq").output().unwrap().stdout.lines()
        .for_each(|line| orphans.push(line.unwrap()));
    if !orphans.is_empty() {
        Command::new("pacman")
            .arg("-Rns").args(orphans)
            .status().unwrap();
        println!("==> Finished.");
    }
    else {
        println!("==> No orphaned dependencies found!");
    }
}

fn clear_paccache() {
    print!("How many package versions would you like to keep? (default: 3) ");
    stdout().flush().unwrap();
    let mut packagenum = String::new();
    stdin().read_line(&mut packagenum).unwrap();

    if packagenum.trim().is_empty() {
        packagenum = "3".to_string();
    }
    else {
        packagenum = packagenum.trim().to_string();
        if packagenum.parse::<u16>().is_err() {
            eprintln!("Invalid number, skipping...");
            return;
        }
    }

    Command::new("paccache")
        .arg("-r").arg("-k").arg(packagenum)
        .status().unwrap();
    println!("==> Finished.");
}

fn clear_sysdlogs() {
    Command::new("journalctl")
        .arg("--disk-usage")
        .status().unwrap();
    print!("Are you sure you would like to clear the logs? (y/N) ");
    stdout().flush().unwrap();

    let mut choice = String::new();
    stdin().read_line(&mut choice).unwrap();
    if choice.trim().to_lowercase().contains("y") {
        choice.clear();
        print!("How many days of logs would you like to keep? (default: 3) ");
        stdout().flush().unwrap();
        stdin().read_line(&mut choice).unwrap();

        if choice.trim().is_empty() {
            Command::new("journalctl")
                .arg("--vacuum-time=3d")
                .status().unwrap();
        }
        else {
            if choice.trim().parse::<u16>().is_err() {
                eprintln!("Invalid number, skipping...");
                return;
            }
            Command::new("journalctl")
                .arg(format!("--vacuum-time={}d", choice.trim()))
                .status().unwrap();
        }
        println!("==> Finished");
    }
    else {
        eprintln!("==> Skipping journal log clearing...");
    }
}

fn main()-> std::io::Result<()> {
    if !Path::new("/etc/pacman.conf").exists() {
        eprintln!("pacman was not detected on this system, are you on an arch-based distro?\nexiting...");
        exit(1);
    }

    if unsafe { getuid() } != 0 {
        eprintln!("[WARNING] !! pactool is not running with root privileges, this may cause issues. !!");
    }

    if !pacman_contrib_is_installed() {
        eprintln!("[WARNING] pacman-contrib is not installed");
        eprintln!("This collection of packages is required for most of pactool's functionality");
        print!("Would you like to install it now? (y/N) ");
        stdout().flush().unwrap();

        let mut input = String::new();
        stdin().read_line(&mut input).unwrap();

        if input.trim().to_lowercase().contains("y") {
            Command::new("pacman")
                .arg("-Sy").arg("pacman-contrib")
                .status().unwrap();
        }
    }

    const VERSION: &str = include_str!("../ver");
    print!("==> pactool version {}

1.) Update all packages
2.) Update all packages and AUR packages
3.) Prune orphaned dependencies
4.) Check for pacdiffs
5.) Clear the pacman cache
6.) Clear the journal logs

choice(s) e.g 1425:
> ", VERSION);

    stdout().flush().unwrap();
    let mut input = String::new();
    stdin().read_line(&mut input).unwrap();

    for choice in input.trim().chars() {
        match choice {
            '1' => {
                println!("==> Updating all packages...");
                Command::new("pacman")
                    .arg("-Syu")
                    .status().unwrap();
                println!("==> Finished.");
            },
            '2' => {
                println!("==> Updating all packages and AUR packages...");
                update_with_aur_helper();
                println!("==> Finished.");
            },
            '3' => {
                println!("==> Pruning orphaned dependencies...");
                prune_orphans();
            },
            '4' => {
                println!("==> Checking for pacdiffs...");
                Command::new("pacdiff")
                    .status().unwrap();
                println!("==> Finished.");
            },
            '5' => {
                println!("==> Clearing the pacman cache...");
                clear_paccache();
            },
            '6' => {
                println!("==> Clearing the journal logs...");
                clear_sysdlogs();
            },
            _ => {
                eprintln!("Invalid choice '{}', skipping...", choice);
            }
        }
    }

    Ok(())
}
