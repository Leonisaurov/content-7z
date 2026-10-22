use std::{io::{BufReader, Read, Write}, process::{Command, Stdio}};

use crate::files::{
    folder::Folder,
    entry::EntryType
};

pub enum ZipManagerType {
    P7ZIP,
    TAR,
    TAR_GZ,
    ZIP,
}

pub struct ZipManager {
    pub t: ZipManagerType,
    pub file_name: String,
    pub output: String,
    pub err: String,
    pub res_code: i32
}

impl ZipManager {
    pub fn process(file_name: &str, password: Option<&str>) -> Self {
        if file_name.ends_with(".tar.gz") {
            return Self::process_tar_gz(file_name)
        }
        if file_name.ends_with(".tar") {
            return Self::process_tar(file_name)
        }
        if file_name.ends_with(".zip") {
            return Self::process_zip(file_name, password)
        }

        let mut args = vec!["l", file_name];
        let password_owned;
        if let Some(password) = password {
            password_owned = password.to_string();
            args.push(&password_owned);
        }

        let res = Command::new("7z")
            .args(args)
            .output();

        match res {
            Err(e) => Self {
                t: ZipManagerType::P7ZIP,
                file_name: file_name.to_string(),
                output: String::new(),
                err: format!("Error al procesar la salida del archivo: {}", e),
                res_code: -1
            },
            Ok(res) => {
                let output = String::from_utf8_lossy(&res.stdout).to_string();
                let err = String::from_utf8_lossy(&res.stderr).to_string();

                Self {
                    t: ZipManagerType::P7ZIP,
                    file_name: file_name.to_string(),
                    output,
                    err,
                    res_code: res.status.code().unwrap_or(-1),
                }
            },
        }
    }

    fn process_tar(file_name: &str) -> Self {
        let res = Command::new("tar")
            .args(vec!["-tf", file_name])
            .output();
        match res {
            Err(e) => Self {
                t: ZipManagerType::TAR,
                file_name: file_name.to_string(),
                output: String::new(),
                err: format!("Error al procesar la salida del archivo: {}", e),
                res_code: -1,
            },
            Ok(res) => {
                let output = String::from_utf8_lossy(&res.stdout).to_string();
                let err = String::from_utf8_lossy(&res.stderr).to_string();

                Self {
                    t: ZipManagerType::TAR,
                    file_name: file_name.to_string(),
                    output,
                    err,
                    res_code: res.status.code().unwrap_or(-1),
                }
            },
        }
    }

    fn process_tar_gz(file_name: &str) -> Self {
        let res = Command::new("tar")
            .args(vec!["-tzf", file_name])
            .output();
        match res {
            Err(e) => Self {
                t: ZipManagerType::TAR_GZ,
                file_name: file_name.to_string(),
                output: String::new(),
                err: format!("Error al procesar la salida del archivo: {}", e),
                res_code: -1,
            },
            Ok(res) => {
                let output = String::from_utf8_lossy(&res.stdout).to_string();
                let err = String::from_utf8_lossy(&res.stderr).to_string();

                Self {
                    t: ZipManagerType::TAR_GZ,
                    file_name: file_name.to_string(),
                    output,
                    err,
                    res_code: res.status.code().unwrap_or(-1),
                }
            },
        }
    }

    fn process_zip(file_name: &str, _password: Option<&str>) -> Self {
        let res = Command::new("unzip")
            .args(vec!["-Z1", file_name])
            .output();
        match res {
            Err(e) => Self {
                t: ZipManagerType::ZIP,
                file_name: file_name.to_string(),
                output: String::new(),
                err: format!("Error al procesar la salida del archivo: {}", e),
                res_code: -1,
            },
            Ok(res) => {
                let output = String::from_utf8_lossy(&res.stdout).to_string();
                let err = String::from_utf8_lossy(&res.stderr).to_string();

                Self {
                    t: ZipManagerType::ZIP,
                    file_name: file_name.to_string(),
                    output,
                    err,
                    res_code: res.status.code().unwrap_or(-1),
                }
            },
        }
    }

    pub fn need_password(file_name: &str) -> bool {
        if file_name.ends_with(".zip") {
            return false;
        }
        if file_name.ends_with(".tar") || file_name.ends_with(".tar.gz") {
            return false;
        }

        let mut command = match Command::new("7z")
            .args(vec!["t", file_name])
            .stdout(Stdio::piped())
            .stdin(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
        {
            Ok(c) => c,
            Err(_) => return false,
        };

        let stdout = match command.stdout.take() {
            Some(s) => s,
            None => { let _ = command.wait(); return false; }
        };
        let stdin = match command.stdin.as_mut() {
            Some(s) => s,
            None => { let _ = command.wait(); return false; }
        };
        let mut reader = BufReader::new(stdout);

        let pattern = "Enter password:";
        let mut buffer = vec![];

        loop {
            let mut byte = [0; 1];
            match reader.read(&mut byte) {
                Ok(0) => break,
                Ok(_) => {
                    buffer.push(byte[0]);
                    let output = String::from_utf8_lossy(&buffer);
                    if output.ends_with('\n') {
                        buffer.clear();
                    } else if output == pattern {
                        let _ = writeln!(stdin, "");
                        let _ = command.kill();
                        return true
                    }
                },
                Err(_) => break,
            }
        }

        let _ = command.wait();
        false
    }

    pub fn get_root(&self) -> Folder {
        match self.t {
            ZipManagerType::TAR | ZipManagerType::TAR_GZ => return Self::get_root_tar(&self.output),
            ZipManagerType::ZIP => return Self::get_root_zip(&self.output),
            _ => {},
        }

        let mut root = Folder::new(".");

        let header = "   Date      Time    Attr         Size   Compressed  Name\n------------------- ----- ------------ ------------  ------------------------\n";
        let start_point = match self.output.find(header) {
            Some(i) => i + header.len(),
            None => return root,
        };
        let clean_output = &self.output[start_point..];
        let lines: Vec<&str> = clean_output.split('\n').collect();

        for line in lines {
            if line.len() < 54 {
                if line.trim().is_empty() {
                    continue;
                }
                // possibly footer
                if line.contains("-----") {
                    break;
                }
                continue;
            }
            if &line[20..25] == "-----" {
                break;
            }
            let name = line[53..].trim();
            if name.is_empty() {
                continue;
            }
            if &line[20..25] == "D...." {
                root.add_entry(name, &EntryType::Folder);
            } else {
                root.add_entry(name, &EntryType::File);
            }
        }

        root
    }

    fn get_root_tar(output: &str) -> Folder {
        let mut root = Folder::new(".");

        for line in output.lines() {
            let name = line.trim();
            if name.is_empty() {
                continue;
            }
            if name.ends_with('/') {
                root.add_entry(&name[..name.len() - 1], &EntryType::Folder);
            } else {
                root.add_entry(name, &EntryType::File);
            }
        }

        root
    }

    fn get_root_zip(output: &str) -> Folder {
        let mut root = Folder::new(".");

        for line in output.lines() {
            let name = line.trim();
            if name.is_empty() {
                continue;
            }
            if name.ends_with('/') {
                root.add_entry(&name[..name.len() - 1], &EntryType::Folder);
            } else {
                root.add_entry(name, &EntryType::File);
            }
        }

        root
    }

    pub fn get_path(&self) -> String {
        self.file_name.to_string()
    }
}
