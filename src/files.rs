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

use std::path::Path;
use std::fs;
use std::process::Command;

pub fn copy_dir_all(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> Result<(), std::io::Error> {
    fs::create_dir_all(&dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ftype = entry.file_type()?;
        if ftype.is_dir() {
            copy_dir_all(entry.path(), dst.as_ref().join(entry.file_name()))?;
        } else {
            fs::copy(entry.path(), dst.as_ref().join(entry.file_name()))?;
        }
    }
    Ok(())
}
pub fn get_file_info(filepath: &str) -> String {
    let fileinfo_raw = Command::new("file")
        .arg("--mime-type")
        .arg(filepath)
        .output()
        .expect("Unable to get type of file");
    String::from(String::from_utf8(fileinfo_raw.stdout).expect("Unable to parse UTF8").split(": ").collect::<Vec<&str>>()[1])
}
pub fn get_file_info_full(filepath: String) -> String {
    let fileinfo_raw = Command::new("file")
        .arg(filepath)
        .output()
        .expect("Unable to get file info");
    String::from_utf8(fileinfo_raw.stdout).expect("Unable to parse UTF8")
}
