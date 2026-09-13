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
use std::process;
use std::process::{Command, exit};
use std::io::prelude::*;
use std::path::Path;
use std::path::PathBuf;
use std::os::unix::fs::PermissionsExt;
// additional dependencies
use reqwest::blocking;
use reqwest::header::USER_AGENT;
use reqwest::StatusCode;
use colored::Colorize;
use xz::read::XzDecoder;
use tar::Archive;
use flate2::read::GzDecoder;
use zip::read::ZipArchive;

fn copy_dir_all(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> Result<(), std::io::Error> {
    fs::create_dir_all(&dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        if ty.is_dir() {
            copy_dir_all(entry.path(), dst.as_ref().join(entry.file_name()))?;
        } else {
            fs::copy(entry.path(), dst.as_ref().join(entry.file_name()))?;
        }
    }
    Ok(())
}
fn get_file_info(filepath: String) -> String {
    let fileinfo_raw = Command::new("file")
        .arg("--mime-type")
        .arg(filepath)
        .output()
        .expect(&format!("{}", "Faliure".red().bold()));
    String::from(String::from_utf8(fileinfo_raw.stdout).expect(&format!("{}", "Faliure".red().bold())).split(": ").collect::<Vec<&str>>()[1])
}
fn get_file_info_full(filepath: String) -> String {
    let fileinfo_raw = Command::new("file")
        .arg(filepath)
        .output()
        .expect(&format!("{}", "Faliure".red().bold()));
    String::from_utf8(fileinfo_raw.stdout).expect(&format!("{}", "Faliure".red().bold()))
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = "/etc/gaia/";
    let install = "/usr/local/bin/";
    let temp = "/tmp";
    fs::create_dir_all(install)?;
    fs::create_dir_all(config)?;
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("{}: No operation specified", "Error".yellow().bold());
        process::exit(3);
    }
    if args[1] != "install" && args[1] != "override" && args[1] != "remove" && args[1] != "setup" && args[1] != "help" && args[1] != "test" {
        println!("{}: {} is not a valid subcommand", "Error".yellow().bold(), args[1]);
        process::exit(3);
    }
    if args[1] == "install" {
        if args.len() < 3 {
            println!("{}: No application specified", "Error".yellow().bold());
            process::exit(3);
        }
        let repo: Vec<&str> = args[2].split("/").collect();
        if repo.len() != 2 {
            println!("{}: {} is not a valid application name. Run 'gaia help install' for more information.", "Error".yellow().bold(), args[2]);
            process::exit(3);
        }
        print!("Preparing to install {} ... ", repo[1]);
        std::io::stdout().flush().unwrap();
        let url = format!("https://api.github.com/repos/{}/releases/latest", args[2].to_lowercase());
        fs::create_dir_all(format!("{config}{}", repo[0].to_lowercase()))?;
        // get data
        let client = blocking::Client::new();
        let response = client.get(&url)
            .header(USER_AGENT, "GAIA Github Automated Installation Application Version 0.1.2")
            .send()?;
        match response.status() {
            StatusCode::OK => println!("{}", "Success".green().bold()),
            StatusCode::NOT_FOUND => {
                println!("{}: That repository doesn't exist (404 not found)", "Faliure".red().bold());
                exit(4);
            },
            _ => {
                println!("{}: Error {}", "Faliure".red().bold(), response.status());
                exit(4);
            },
        }
        let rawjson = &response.text()?;
        let json = json::parse(rawjson)?;
        let mut i = 0;
        while !json["assets"][i].is_null() {
            let asset = &json["assets"][i]["browser_download_url"].to_string();
            let fname = Path::new(asset).file_name().expect("Unable to parse file name").display().to_string();
            print!("Downloading file {fname} ... ");
            std::io::stdout().flush().unwrap();
            let response = client.get(asset)
                .header(USER_AGENT, "GAIA Github Automated Installation Application Version 0.1.2")
                .send()?;
            match response.status() {
                StatusCode::OK => {},
                StatusCode::NOT_FOUND => {
                    println!("{}: File doesn't exist (404 not found)", "Faliure".red().bold());
                    exit(4);
                },
                _ => {
                    println!("{}: Error {}", "Faliure".red().bold(), response.status());
                    exit(4);
                },
            }
            {
                let response_bytes = response.bytes()?;
                let mut downloaded_file = File::create(format!("{temp}/{fname}"))?;
                downloaded_file.write_all(&response_bytes)?;
            }
            println!("{}", "Success".green().bold());
            let mut files: Vec<String> = vec![format!("{temp}/{fname}")];
            while files.len() > 0 {
                let file_name_pretty = Path::new(&files[0]).file_name().unwrap().display().to_string();
                print!("Getting type of file {file_name_pretty} ... ");
                std::io::stdout().flush().unwrap();
                let filetype = get_file_info(files[0].clone());
                if filetype == "application/x-xz\n" {
                    println!("{}", "Success".green().bold());
                    print!("Extracting XZ file {file_name_pretty} ... ");
                    std::io::stdout().flush().unwrap();
                    let mut xzbytes: Vec<u8> = vec![];
                    let mut xzfile = File::open(format!("{temp}/{file_name_pretty}"))?;
                    xzfile.read_to_end(&mut xzbytes)?;
                    let mut decompressed = XzDecoder::new(&*xzbytes);
                    let mut bytes: Vec<u8> = vec![];
                    decompressed.read_to_end(&mut bytes)?;
                    let mut extracted_file = File::create(format!("{}_extracted", files[0]))?;
                    extracted_file.write_all(&bytes)?;
                    files.push(format!("{}_extracted", files[0]));
                    println!("{}", "Success".green().bold());
                    files.remove(0);
                    continue;
                }
                if filetype == "application/gzip\n" {
                    println!("{}", "Success".green().bold());
                    print!("Extracting GZip file {file_name_pretty} ... ");
                    std::io::stdout().flush().unwrap();
                    let gzfile = File::open(format!("{temp}/{file_name_pretty}"))?;
                    let mut decoded = GzDecoder::new(gzfile);
                    let mut bytes: Vec<u8> = vec![];
                    decoded.read_to_end(&mut bytes)?;
                    let mut extracted_file = File::create(format!("{}_extracted", files[0]))?;
                    extracted_file.write_all(&bytes)?;
                    files.push(format!("{}_extracted", files[0]));
                    println!("{}", "Success".green().bold());
                    files.remove(0);
                    continue;
                }
                if filetype == "application/x-tar\n" {
                    println!("{}", "Success".green().bold());
                    print!("Extracting tar file {file_name_pretty} ... ");
                    std::io::stdout().flush().unwrap();
                    fs::create_dir_all(format!("{temp}/{file_name_pretty}_extraction_dest"))?;
                    let tar = File::open(format!("{temp}/{file_name_pretty}"))?;
                    let mut archive = Archive::new(tar);
                    archive.unpack(format!("{temp}/{file_name_pretty}_extraction_dest"))?;
                    println!("{}", "Success".green().bold());
                    print!("Reading contents of extracted directory ... ");
                    std::io::stdout().flush().unwrap();
                    let files_again = fs::read_dir(format!("{temp}/{file_name_pretty}_extraction_dest"))?;
                    for f in files_again {
                        let file = f?.path().display().to_string();
                        let finfo = get_file_info_full(file.clone());
                        // im going fucking insane
                        let files2electricboogaloo = fs::read_dir(format!("{temp}/{file_name_pretty}_extraction_dest"))?;
                        if finfo.contains("directory") && files2electricboogaloo.count() == 1 {
                            let files_again_again = fs::read_dir(format!("{}", file))?;
                            for ff in files_again_again {
                                files.push(ff?.path().display().to_string());
                            }
                            break;
                        } else {
                            files.push(file.clone());
                        }
                    }
                    println!("{}", "Success".green().bold());
                    files.remove(0);
                    continue;
                }
                if filetype == "application/zip\n" {
                    println!("{}", "Success".green().bold());
                    print!("Extracting zip file {file_name_pretty} ... ");
                    std::io::stdout().flush().unwrap();
                    fs::create_dir_all(format!("{temp}/{file_name_pretty}_extraction_dest"))?;
                    let zip = File::open(format!("{temp}/{file_name_pretty}"))?;
                    let mut archive = ZipArchive::new(zip)?;
                    archive.extract_unwrapped_root_dir(format!("{temp}/{file_name_pretty}_extraction_dest"), zip::read::root_dir_common_filter)?;
                    println!("{}", "Success".green().bold());
                    print!("Reading contents of extracted directory ... ");
                    std::io::stdout().flush().unwrap();
                    let files_again = fs::read_dir(format!("{temp}/{file_name_pretty}_extraction_dest"))?;
                    for f in files_again {
                        let file = f?.path().display().to_string();
                        let finfo = get_file_info_full(file.clone());
                        // im going fucking insane
                        let files2electricboogaloo = fs::read_dir(format!("{temp}/{file_name_pretty}_extraction_dest"))?;
                        if finfo.contains("directory") && files2electricboogaloo.count() == 1 {
                            let files_again_again = fs::read_dir(format!("{}", file))?;
                            for ff in files_again_again {
                                files.push(ff?.path().display().to_string());
                            }
                            break;
                        } else {
                            files.push(file.clone());
                        }
                    }
                    println!("{}", "Success".green().bold());
                    files.remove(0);
                    continue;
                }
                if filetype.contains("executable") {
                    let filetype_full = get_file_info_full(files[0].clone());
                    if filetype_full.contains("ELF") && filetype_full.contains("x86") {
                        println!("{}, {file_name_pretty} is a valid executable", "Success".green().bold());
                        print!("Installing to {install} ... ");
                        std::io::stdout().flush().unwrap();
                        let mut configfile = File::create(format!("{config}{}/{}", repo[0], repo[1]))?;
                        let mut configinfo = String::new();
                        let mut path = PathBuf::from(files[0].clone());
                        path.pop();
                        if path == PathBuf::from(&temp) {
                            fs::copy(format!("{temp}/{file_name_pretty}"), format!("{install}{file_name_pretty}"))?;
                            configinfo = format!("{configinfo}{file_name_pretty}\n");
                            let perms = fs::Permissions::from_mode(0o555);
                            let installed = File::open(format!("{install}{file_name_pretty}"))?;
                            installed.set_permissions(perms)?;
                        } else {
                            let files_one_last_time = fs::read_dir(format!("{}", path.display()))?;
                            for file_one_last_time in files_one_last_time {
                                let oh_my_fucking_god = file_one_last_time?.path();
                                let pretty_name = Path::new(&oh_my_fucking_god).file_name().unwrap().display().to_string();
                                if oh_my_fucking_god.is_dir() {
                                    copy_dir_all(oh_my_fucking_god, format!("{install}{pretty_name}"))?;
                                } else {
                                    fs::copy(oh_my_fucking_god, format!("{install}{pretty_name}"))?;
                                    let perms = fs::Permissions::from_mode(0o555);
                                    let installed = File::open(format!("{install}{file_name_pretty}"))?;
                                    installed.set_permissions(perms)?;
                                }
                                configinfo = format!("{configinfo}{pretty_name}\n");
                            }
                        }
                        println!("{}", "Success".green().bold());
                        std::io::stdout().flush().unwrap();
                        print!("Writing configuration ... ");
                        configfile.write_all(configinfo.as_bytes())?;
                        println!("{}", "Success".green().bold());
                        println!("{} {}", "Installation complete, overall".bold(), "Success".green().bold());
                        exit(0);
                    } else {
                        println!("{} {file_name_pretty} is not a valid executable", "Done but".cyan().bold());
                        files.remove(0);
                        continue;
                    }
                }
                print!("{} no operation can be performed on file of type {filetype}", "Done but".cyan().bold());
                files.remove(0);
            }
            i += 1;
        }
        println!("{} {}: No valid executable found", "Installation complete, overall".bold(), "Faliure".red().bold());
    }
    if args[1] == "remove" {
        if args.len() < 3 {
            println!("{}: No application specified", "Error".yellow().bold());
            process::exit(3);
        }
        let repo: Vec<&str> = args[2].split("/").collect();
        println!("{:#?}", repo);
        let remove = args[2].to_lowercase();
        print!("Getting {}'s file list ... ", repo[1]);
        std::io::stdout().flush().unwrap();
        let pathstr = format!("{}{}", config, remove);
        let path = Path::new(&pathstr);
        if path.exists() {
            let mut file = File::open(pathstr)?;
            let mut contents = String::new();
            file.read_to_string(&mut contents)?;
            println!("{}", "Success".green().bold());
            let split: Vec<&str> = contents.split("\n").collect();
            for i in split {
                if i != "" {
                    print!("Removing {} ... ", i);
                    std::io::stdout().flush().unwrap();
                    if !fs::exists(install.to_owned() + i).expect(&format!("{}: Could not determine whether the file exists", "!FATAL!".yellow().bold())) {
                        println!("{}: {} does not exist. ", "Skipped".magenta().bold(), i);
                        continue;
                    } else if fs::metadata(install.to_owned() + i)?.is_file() {
                        fs::remove_file(format!("{}{}", install, i))?;
                    } else if fs::metadata(install.to_owned() + i)?.is_dir() {
                        fs::remove_dir_all(format!("{}{}", install, i))?;
                    }
                    println!("{}", "Success".green().bold());
                }
            }
            print!("Removing {}'s file list ... ", repo[1]);
            std::io::stdout().flush().unwrap();
            fs::remove_file(format!("{}{}", config, remove))?;
            println!("{}", "Success".green().bold());
            println!("{} {}", "Removal complete, overall".bold(), "Success".green().bold());
        } else {
            println!("{}: Unable to get {}'s file list", "Faliure".red().bold(), repo[1]);
        }
    }
    if args[1] == "setup" {
        if args.len() > 2 {
            println!("{}: Subcommand setup needs no additional arguments", "Error".yellow().bold());
            process::exit(3);
        }
        print!("Moving executable to /usr/local/bin ... ");
        std::io::stdout().flush().unwrap();
        let currentpath = env::current_exe()?.display().to_string();
        fs::rename(currentpath, "/usr/local/bin/gaia")?;
        println!("{}", "Success".green().bold());
        print!("Creating confiuration ... ");
        std::io::stdout().flush().unwrap();
        fs::create_dir_all("/etc/gaia/ralyrona/")?;
        let file = File::create("/etc/gaia/ralyrona/gaia");
        file?.write_all(b"gaia")?;
        println!("{}", "Success".green().bold());
        println!("The Github Automated Installation Application is now installed on your device! Run 'gaia help' for a list of subcommands");
    }
    if args[1] == "help" {
        if args.len() < 3 {
            println!("Github Automated Installation Application v0.1.2
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
                println!("GAIA install subcommand
Usage: sudo gaia <install or override> <account name>/<repository name>

The install subcommand is used to install applications. To specify the
application to install, you must enter the name of the account which
owns the repository, followed by a forward slash (/), followed by the
name of the repository. For example, you could run
'sudo gaia install ralyrona/writeST' to install writeST, my image editor,
or you could run 'sudo gaia install fish-shell/fish-shell' to install
Fish, the friendly interactive shell.
Please note that to install an application, the targeted repository
must have at least one release and its latest release must have at least
one asset that is not source code.");
            }
            if args[2] == "remove" {
                println!("GAIA remove subcommand
Usage: sudo gaia remove <account name>/<repository name>

Remove removes applications and any configuration data that GAIA created
for them.");
            }
        }
    }
    Ok(())
}
