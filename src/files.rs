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
