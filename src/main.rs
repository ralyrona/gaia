/* Github Automated Installation Application
Copyright (C) 2026 Ralyrona

This program is free software: you can redistribute it and/or modify
it under the terms of the GNU General Public License as published by
the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU General Public License for more details. */

// in standard crate
use std::fs;
use std::fs::File;
use std::env;
use std::io::prelude::*;
// archive extraction
// status macros and functions
use gaia::{fail, err, info};
use gaia::status::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // fs::create_dir_all(install)?;
    // fs::create_dir_all(config)?;
    // fs::create_dir_all(temp)?;
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        err!("No operation specified");
    }
    match args[1].as_str() {
        "install" => {
            if args.len() < 3 {
                err!("No application specified for subcommand 'install'");
            }
            let repo: Vec<&str> = args[2].split("/").collect();
            if repo.len() != 2 {
                err!("{} is not a valid application name. Run 'gaia help install' for more information.", args[2]);
            }
            match gaia::install(&repo[0], &repo[1]) {
                Ok(_) => finish("Installation"),
                Err(e) => fail!("{e}"),
            }
        },
        "remove" => {
            if args.len() < 3 {
                err!("No application specified");
            }
            let repo: Vec<&str> = args[2].split("/").collect();
            match gaia::remove(&repo[0], &repo[1]) {
                Ok(_) => finish("Removal"),
                Err(e) => fail!("{e}"),
            }
        },
        "setup" => {
            if args.len() > 2 {
                err!("Subcommand setup needs no additional arguments");
            }
            info!("Moving executable to /usr/local/bin");
            let currentpath = env::current_exe()?.display().to_string();
            fs::copy(currentpath, "/usr/local/bin/gaia")?;
            succeed();
            info!("Creating confiuration");
            fs::create_dir_all("/etc/gaia/ralyrona/")?;
            let file = File::create("/etc/gaia/ralyrona/gaia");
            file?.write_all(b"gaia")?;
            succeed();
            println!("The Github Automated Installation Application is now installed on your device! Run 'gaia help' for a list of subcommands");
        },
        _ => err!("Subcommand {} is not recognised", args[1]),
    }
    if args[1] == "help" {
        if args.len() < 3 {
            eprintln!("Github Automated Installation Application v0.1.2
Usage: gaia <subcommand> <arguments>

GAIA is a commandline installation application. It is designed to automate
the installation of files from the extremely large open-source application
sharing website known as Github.

Subcommands:
install     install an application
remove      remove an application
help        show this help, or show more detailed information about a
            specific subcommand

Run 'gaia help <subcommand>' for more information about a specific subcommand.");
        } else {
            if args[2] == "install" {
                eprintln!("GAIA install subcommand
Usage: gaia install <account name>/<repository name>

The install subcommand is used to install applications. To specify the
application to install, you must enter the name of the account which
owns the repository, followed by a forward slash (/), followed by the
name of the repository. For example, you could run
'gaia install ralyrona/writeST' to install writeST, my image editor,
or you could run 'gaia install fish-shell/fish-shell' to install Fish,
the friendly interactive shell.
Please note that to install an application, the targeted repository
must have at least one release and its latest release must have at least
one asset that is not source code.");
            }
            if args[2] == "remove" {
                eprintln!("GAIA remove subcommand
Usage: gaia remove <account name>/<repository name>

Remove removes applications and any configuration data that GAIA created
for them.");
            }
        }
    }
    Ok(())
}
