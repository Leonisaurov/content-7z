use std::process::Command;

use crate::files::{
    folder::Folder,
    entry::EntryType
};

pub enum ZipManagerType {
    P7ZIP,
    TAR,
}

pub struct ZipManager {
    pub t: ZipManagerType,
    pub file_name: String,
    pub output: String,
    pub err: String,
    pub res_code: i32
}

impl ZipManager {
    pub fn process(file_name: &str) -> Self {
        if file_name.ends_with(".tar.gz") {
            return Self::process_tar(file_name)
        }

        let res = Command::new("7z")
            .args(vec!["l", file_name])
            .output();
        match res {
            Err(_) => Self {
                t: ZipManagerType::P7ZIP,
                file_name: file_name.to_string(),
                output: String::new(),
                err: String::from("Error al procesar la salida del archivo"),
                res_code: -1
            },
            Ok(res) => {
                let output = std::str::from_utf8(&res.stdout).unwrap();
                let err = std::str::from_utf8(&res.stderr).unwrap();

                Self {
                    t: ZipManagerType::P7ZIP,
                    file_name: file_name.to_string(),
                    output: String::from(output),
                    err: String::from(err),
                    res_code: res.status.code().unwrap(),
                }
            },
        }
    }

    fn process_tar(file_name: &str) -> Self {
        let res = Command::new("tar")
            .args(vec!["-tf", file_name])
            .output();
        match res {
            Err(_) => Self {
                t: ZipManagerType::TAR,
                file_name: file_name.to_string(),
                output: String::new(),
                err: String::from("Error al procesar la salida del archivo"),
                res_code: -1,
            },
            Ok(res) => {
                let output = std::str::from_utf8(&res.stdout).unwrap();
                let err = std::str::from_utf8(&res.stderr).unwrap();

                Self {
                    t: ZipManagerType::TAR,
                    file_name: file_name.to_string(),
                    output: String::from(output),
                    err: String::from(err),
                    res_code: res.status.code().unwrap(),
                }
            },
        }
    }

    pub fn get_root(&self) -> Folder {
        match self.t {
            ZipManagerType::TAR => return Self::get_root_tar(self),
            _ => {},
        }

        let mut root = Folder::new(".");

        let start_point: usize = self.output.find("   Date      Time    Attr         Size   Compressed  Name\n------------------- ----- ------------ ------------  ------------------------\n").expect("The content isn't be found") + "   Date      Time    Attr         Size   Compressed  Name\n------------------- ----- ------------ ------------  ------------------------\n".len();
        let clean_output = &self.output[start_point..];
        let lines: Vec<&str> = clean_output.split("\n").collect();

        for line in lines {
            if &line[20..25].to_string() == "-----" {
                break;
            }
            if &line[20..25].to_string() == "D...." {
                root.add_entry(&line[53..].to_string(), &EntryType::Folder);
            } else {
                root.add_entry(&line[53..].to_string(), &EntryType::File);
            }
        }

        root
    }

    pub fn get_root_tar(&self) -> Folder {
        let mut root = Folder::new(".");

        let mut lines: Vec<&str> = self.output.split("\n").collect();
        lines.pop();

        for line in lines {
            if line.ends_with("/") {
                root.add_entry(&line[..line.len() - 1], &EntryType::Folder);
            } else {
                root.add_entry(line, &EntryType::File);
            }
        }

        root
    }

    pub fn get_path(&self) -> String {
        self.file_name.to_string()
    }
}

