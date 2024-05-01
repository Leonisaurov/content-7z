use std::process::Command;
use crate::files::{
    folder::Folder,
    entry::EntryType
};

pub struct ZipManager {
    pub output: String,
    pub err: String,
    pub res_code: i32
}

impl ZipManager {
    pub fn process(file_name: &str) -> Self {
        let res = Command::new("7z")
            .args(vec!["l", file_name])
            .output();
        match res {
            Err(_) => Self {
                output: String::new(),
                err: String::from("Error al procesar la salida del archivo"),
                res_code: -1
            },
            Ok(res) => {
                let output = std::str::from_utf8(&res.stdout).unwrap();
                let err = std::str::from_utf8(&res.stderr).unwrap();

                Self {
                    output: String::from(output),
                    err: String::from(err),
                    res_code: res.status.code().unwrap(),
                }
            },
        }
    }

    pub fn get_root(&self) -> Folder {
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

    pub fn get_path(&self) -> String {
        let path_start = self.output.find("Path = ").expect("No path") + 7;
        let path_end = self.output.find("\nType = ").expect("No path end");

        if let Some(path) = self.output.get(path_start..path_end) {
            String::from(path)
        } else {
            String::from("Unreacheable")
        }
    }
}

