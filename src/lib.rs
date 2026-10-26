pub mod status;
pub mod files;

use std::fs;
use std::fs::File;
use std::path::Path;
use std::path::PathBuf;
use std::sync::LazyLock;
use std::io::{Read, Write};
use std::os::unix::fs::PermissionsExt;
use reqwest::blocking;
use reqwest::header::USER_AGENT;
use reqwest::StatusCode;
use xz::read::XzDecoder;
use tar::Archive;
use flate2::read::GzDecoder;
use zip::read::ZipArchive;
use crate::status::{succeed, handle_result};
use crate::files::*;

const CONFIG_DIR: &str = "/etc/gaia/";
const INSTALL_DIR: &str = "/usr/local/bin";
const TEMP_DIR: &str = "/tmp";
const VERSION: &str = "0.2.1";
static USER_AGENT_NAME: LazyLock<String> = LazyLock::new(|| {
    "GAIA Github Automated Installation Application Version ".to_owned() + VERSION
});

pub fn install(account: &str, repo: &str) -> Result<(), InstallError> {
    use InstallError::*;
    let full_repo_name = format!("{account}/{repo}");
    info!("Preparing to install {repo}");
    let url = format!("https://api.github.com/repos/{}/releases/latest", full_repo_name.to_lowercase());
    fs::create_dir_all(format!("{CONFIG_DIR}{}", account.to_lowercase()))?;
    // get data
    let client = blocking::Client::new();
    let response = client.get(&url)
    .header(USER_AGENT, &*USER_AGENT_NAME)
    .send()?;
    match response.status() {
        StatusCode::OK => succeed(),
        StatusCode::NOT_FOUND => return Err(RepoDoesntExist(full_repo_name)),
        _ => return Err(HttpError(response.status())),
    }
    let rawjson = &response.text()?;
    let json = json::parse(rawjson)?;
    let mut i = 0;
    while !json["assets"][i].is_null() {
        let asset = &json["assets"][i]["browser_download_url"].to_string();
        let fname = Path::new(asset).file_name().ok_or(CannotParseFileName)?.display().to_string();
        info!("Downloading file {fname}");
        let response = client.get(asset)
        .header(USER_AGENT, &*USER_AGENT_NAME)
        .send()?;
        match response.status() {
            StatusCode::OK => {},
            StatusCode::NOT_FOUND => {
                return Err(CannotGetFile);
            },
            _ => {
                return Err(HttpError(response.status()));
            },
        }
        {
            let response_bytes = response.bytes()?;
            let mut downloaded_file = File::create(format!("{TEMP_DIR}/{fname}"))?;
            downloaded_file.write_all(&response_bytes)?;
        }
        succeed();
        let mut files: Vec<String> = vec![format!("{TEMP_DIR}/{fname}")];
        while files.len() > 0 {
            let file_name_pretty = Path::new(&files[0]).file_name().unwrap().display().to_string();
            info!("Getting type of file {file_name_pretty}");
            let filetype = get_file_info(&files[0]);
            match filetype.as_str() {
                "application/x-xz\n" => {
                    succeed();
                    info!("Extracting XZ file {file_name_pretty}");
                    let mut xzbytes: Vec<u8> = vec![];
                    let mut xzfile = File::open(format!("{TEMP_DIR}/{file_name_pretty}"))?;
                    xzfile.read_to_end(&mut xzbytes)?;
                    let mut decompressed = XzDecoder::new(&*xzbytes);
                    let mut bytes: Vec<u8> = vec![];
                    decompressed.read_to_end(&mut bytes)?;
                    let mut extracted_file = File::create(format!("{}_extracted", files[0]))?;
                    extracted_file.write_all(&bytes)?;
                    files.push(format!("{}_extracted", files[0]));
                    succeed();

                },
                "application/gzip\n" => {
                    succeed();
                    info!("Extracting GZip file {file_name_pretty}");
                    let gzfile = File::open(format!("{TEMP_DIR}/{file_name_pretty}"))?;
                    let mut decoded = GzDecoder::new(gzfile);
                    let mut bytes: Vec<u8> = vec![];
                    decoded.read_to_end(&mut bytes)?;
                    let mut extracted_file = File::create(format!("{}_extracted", files[0]))?;
                    extracted_file.write_all(&bytes)?;
                    files.push(format!("{}_extracted", files[0]));
                    succeed();
                },
                "application/x-tar\n" => {
                    succeed();
                    info!("Extracting tar file {file_name_pretty}");
                    fs::create_dir_all(format!("{TEMP_DIR}/{file_name_pretty}_extraction_dest"))?;
                    let tar = File::open(format!("{TEMP_DIR}/{file_name_pretty}"))?;
                    let mut archive = Archive::new(tar);
                    archive.unpack(format!("{TEMP_DIR}/{file_name_pretty}_extraction_dest"))?;
                    succeed();
                    info!("Reading contents of extracted directory");
                    let files_again = fs::read_dir(format!("{TEMP_DIR}/{file_name_pretty}_extraction_dest"))?;
                    for f in files_again {
                        let file = f?.path().display().to_string();
                        let finfo = get_file_info_full(file.clone());
                        // im going fucking insane
                        let files2electricboogaloo = fs::read_dir(format!("{TEMP_DIR}/{file_name_pretty}_extraction_dest"))?;
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
                    succeed();
                },
                "application/zip\n" => {
                    succeed();
                    info!("Extracting zip file {file_name_pretty}");
                    fs::create_dir_all(format!("{TEMP_DIR}/{file_name_pretty}_extraction_dest"))?;
                    let zip = File::open(format!("{TEMP_DIR}/{file_name_pretty}"))?;
                    let mut archive = ZipArchive::new(zip)?;
                    archive.extract_unwrapped_root_dir(format!("{TEMP_DIR}/{file_name_pretty}_extraction_dest"), zip::read::root_dir_common_filter)?;
                    succeed();
                    info!("Reading contents of extracted directory");
                    let files_again = fs::read_dir(format!("{TEMP_DIR}/{file_name_pretty}_extraction_dest"))?;
                    for f in files_again {
                        let file = f?.path().display().to_string();
                        let finfo = get_file_info_full(file.clone());
                        // im going fucking insane
                        let files2electricboogaloo = fs::read_dir(format!("{TEMP_DIR}/{file_name_pretty}_extraction_dest"))?;
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
                    succeed();
                }
                _ => if filetype.contains("executable") {
                    let filetype_full = get_file_info_full(files[0].clone());
                    if filetype_full.contains("ELF") && filetype_full.contains("x86") {
                        succeed();
                        info!("Installing to {INSTALL_DIR}");
                        let mut configfile = File::create(format!("{CONFIG_DIR}{}/{}", account, repo))?;
                        let mut configinfo = String::new();
                        let mut path = PathBuf::from(files[0].clone());
                        path.pop();
                        if path == PathBuf::from(&TEMP_DIR) {
                            fs::copy(format!("{TEMP_DIR}/{file_name_pretty}"), format!("{INSTALL_DIR}/{file_name_pretty}"))?;
                            configinfo = format!("{configinfo}{file_name_pretty}\n");
                            let perms = fs::Permissions::from_mode(0o555);
                            let installed = File::open(format!("{INSTALL_DIR}/{file_name_pretty}"))?;
                            installed.set_permissions(perms)?;
                        } else {
                            // WHAT THE FUCK DID I DO
                            let files_one_last_time = fs::read_dir(format!("{}", path.display()))?;
                            for file_one_last_time in files_one_last_time {
                                let oh_my_fucking_god = file_one_last_time?.path();
                                let pretty_name = Path::new(&oh_my_fucking_god).file_name().unwrap().display().to_string();
                                if oh_my_fucking_god.is_dir() {
                                    handle_result(copy_dir_all(oh_my_fucking_god, format!("{INSTALL_DIR}/{pretty_name}")));
                                } else {
                                    handle_result(fs::copy(oh_my_fucking_god, format!("{INSTALL_DIR}/{pretty_name}")));
                                    let perms = fs::Permissions::from_mode(0o555);
                                    let installed = File::open(format!("{INSTALL_DIR}/{file_name_pretty}"))?;
                                    installed.set_permissions(perms)?;
                                }
                                configinfo = format!("{configinfo}{pretty_name}\n");
                            }
                        }
                        succeed();
                        info!("Updating configuration");
                        configfile.write_all(configinfo.as_bytes())?;
                        succeed();
                        return Ok(());
                    } else {
                        done_but!("{file_name_pretty} is not a valid executable\n");
                    }
                } else {
                    done_but!("no operation can be performed on file of type {filetype}");
                }
            }
            files.remove(0);
        }
        i += 1;
    }
    Err(NoValidExec)
}

pub fn remove(account: &str, repo: &str) -> Result<(), RemoveError> {
    use RemoveError::*;
    let full_repo_name = format!("{account}/{repo}");
    info!("Getting {}'s file list", repo);
    let pathstr = format!("{CONFIG_DIR}{full_repo_name}");
    let path = Path::new(&pathstr);
    if path.exists() {
        let mut file = File::open(pathstr)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        succeed();
        let entries: Vec<&str> = contents.split("\n").collect();
        for entry in entries {
            if entry != "" {
                info!("Removing {entry}");
                if !fs::exists(INSTALL_DIR.to_owned() + "/" + entry)? {
                    done_but!("{} does not exist", entry);
                    continue;
                } else if fs::metadata(INSTALL_DIR.to_owned() + "/" + entry)?.is_file() {
                    fs::remove_file(format!("{INSTALL_DIR}/{entry}"))?;
                } else if fs::metadata(INSTALL_DIR.to_owned() + "/" + entry)?.is_dir() {
                    fs::remove_dir_all(format!("{INSTALL_DIR}/{entry}"))?;
                }
                succeed();
            }
        }
        info!("Removing {}'s file list", repo);
        fs::remove_file(format!("{CONFIG_DIR}{full_repo_name}"))?;
        succeed();
        return Ok(());
    } else {
        return Err(CannotGetFileList(repo.to_string()));
    }
}

use std::convert::From;
use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub enum RemoveError {
    CannotGetFileList(String),
    IoError(std::io::Error),
}
impl From<std::io::Error> for RemoveError {
    fn from(err: std::io::Error) -> Self {
        RemoveError::IoError(err)
    }
}
impl fmt::Display for RemoveError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use RemoveError::*;
        match self {
            CannotGetFileList(repo) => write!(f, "Unable to get {repo}'s file list"),
            IoError(err) => write!(f, "Encountered IO error: {err}"),
        }
    }
}
impl Error for RemoveError {}

#[derive(Debug)]
pub enum InstallError {
    CannotParseFileName,
    RepoDoesntExist(String),
    CannotGetFile,
    HttpError(StatusCode),
    NoValidExec,
    IoError(std::io::Error),
    ZipError(zip::result::ZipError),
    ReqwestError(reqwest::Error),
    JsonError,
}
impl From<std::io::Error> for InstallError {
    fn from(err: std::io::Error) -> Self {
        InstallError::IoError(err)
    }
}
impl From<zip::result::ZipError> for InstallError {
    fn from(err: zip::result::ZipError) -> Self {
        InstallError::ZipError(err)
    }
}
impl From<reqwest::Error> for InstallError {
    fn from(err: reqwest::Error) -> Self {
        InstallError::ReqwestError(err)
    }
}
impl From<json::Error> for InstallError {
    fn from(_: json::Error) -> Self {
        InstallError::JsonError
    }
}
impl fmt::Display for InstallError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use InstallError::*;
        match self {
            CannotParseFileName => write!(f, "Unable to parse file name"),
            RepoDoesntExist(repo) => write!(f, "Repository {repo} doesn't exist, or it doesn't have any releases (404 not found)"),
            CannotGetFile => write!(f, "The file does not exist (404 not found)"),
            HttpError(code) => write!(f, "HTTP error {code}"),
            NoValidExec => write!(f, "No valid executable found"),
            IoError(err) => write!(f, "Encountered IO error: {err}"),
            ZipError(err) => write!(f, "Unable to unzip files, encountered {err}"),
            ReqwestError(err) => write!(f, "Failed to GET data due to {err}"),
            JsonError => write!(f, "Failed to parse Json\nThis likely means that GAIA's understanding of the Github API is out of date\nIt is very unlikely that you will be able to fix this issue, please submit a bug report at <https://github.com/ralyrona/gaia/issues/new>"),
        }
    }
}
impl Error for InstallError {}
