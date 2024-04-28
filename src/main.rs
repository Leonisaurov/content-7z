use std::process::{Command, exit};
use std::env::args;

#[derive(Debug)]
enum Entry {
    File(String),
    Folder(Folder),
}

#[derive(Clone, Debug)]
enum EntryType {
    File,
    Folder
}

#[derive(Debug)]
struct Folder {
    name: String,
    content: Vec<Entry>,
}

impl Folder {
    fn new(name: &str) -> Self {
        Self {
            name: String::from(name),
            content: Vec::new()
        }
    }

    fn add_file(&mut self, file_name: &str) {
        self.content.push(Entry::File(file_name.to_string()));
    }

    fn add_folder(&mut self, folder: Folder) {
        self.content.push(Entry::Folder(folder));
    }

    fn get_folder(&mut self, folder_name: &str) -> Option<&mut Folder> {
        for entry in &mut self.content {
            if let Entry::Folder(folder) = entry {
                if folder.name == folder_name {
                    return Some(folder)
                }
            }
        }
        None
    }

    fn contain_entry(&mut self, entry_name: &str) -> bool {
        for entry in &self.content {
            match entry {
                Entry::File(file_name) => {
                    if file_name == entry_name {
                        return true
                    }
                },
                Entry::Folder(folder) => {
                    if folder.name == entry_name {
                        return true
                    }
                },
            }
        }
        false
    }

    fn add_entry(&mut self, entry: &str, file_type: &EntryType) {
        //println!("Source Entry: {}", entry);
        for i in 0..entry.len() + 1 {
            if let Some(character) = entry.get(i..i+1) {
                if character == "/" {
                    if let Some(path) = entry.get(0..i) {
                        if self.contain_entry(path) {
                            if let Some(sub_path) = entry.get(i+1..entry.len()) {
                                if let Some(folder) = self.get_folder(path) {
                                    folder.add_entry(sub_path, file_type);
                                    break;
                                }
                            }
                            //println!("In the Main Folder {}.", path);
                        }
                        let mut new_entry = Folder::new(path);
                        //println!("Main Folder {} Added.", path);
                        if let Some(sub_path) = entry.get(i+1..entry.len()) {
                            new_entry.add_entry(sub_path, file_type);
                            //println!("Se pudo?");
                        }
                        //println!("In the Main Folder {}.", path);
                        self.add_folder(new_entry);
                        
                    }
                    break;
                }
            } else {
                if let Some(path) = entry.get(0..i) {
                    if let EntryType::Folder = file_type {
                        //println!("Main Folder {} Added withouth follow.", path);
                        self.add_folder(Folder::new(path))
                    } else {
                        //println!("File {} added.", path);
                        self.add_file(path);
                    }
                } else {
                    //println!("No se pudo?");
                }
                break;
            }
        }
    }
    
    fn strace(&self, indent: usize) {
        let indent_char = " ".repeat(indent);
        println!("{}Folder: {}", " ".repeat(indent - 1) + "└┬", self.name);
        let mut i = 0;
        for entry in &self.content {
            match entry {
                Entry::File(file_name) => {
                    if i + 1 < self.content.len() {
                        println!("{} ├file: {}", indent_char, file_name);
                    } else {
                        println!("{} └file: {}", indent_char, file_name);
                    }
                },
                Entry::Folder(folder) => folder.strace(indent + 1),
            }
            i += 1;
        }
    }

    fn print(&self) {
        self.strace(1);
    }
}

fn get_root(output: String) -> Folder {
    let mut root = Folder::new(".");

    let mut start_point: usize = output.find("   Date      Time    Attr         Size   Compressed  Name\n------------------- ----- ------------ ------------  ------------------------\n").expect("The content isn't be found") + "   Date      Time    Attr         Size   Compressed  Name\n------------------- ----- ------------ ------------  ------------------------\n".len();
    let mut name_indicate: u8 = 0;
    let mut file_type: &EntryType = &EntryType::File;

    loop {
        if let Some(character) = output.get(start_point..(start_point + 1)) {
            //println!("Found: {}", character);
            if character == " " || character == "\t" || character == "\n" {
                //println!("Relleno");
                start_point += 1;
                continue;
            }

            let init_point = start_point;
            loop {
                if let Some(character) = output.get(start_point..(start_point + 1)) {
                    //println!("Found in word: {}", character);
                    if character == " " || character == "\t" || character == "\n" {
                        //println!("Relleno final");
                        break;
                    }
                    start_point += 1;
                } else {
                    break;
                }

            }

            if name_indicate == 5 {
                if let Some(path) = output.get(init_point..start_point) {
                    //println!("Added {} To Root, was a {:?}", path, file_type);
                    root.add_entry(path, file_type);
                    name_indicate = 0;
                } else {
                    break;
                }
            } else if name_indicate == 2 {
                if let Some(meta_data) = output.get(init_point..start_point) {
                    if meta_data == "D...." {
                        file_type = &EntryType::Folder;
                    } else {
                        file_type = &EntryType::File;
                    }
                }
                name_indicate += 1;
            } else {
                if name_indicate == 0 { 
                    if let Some(path) = output.get(init_point..start_point) {
                        if path == "-------------------" {
                            break;
                        }
                    }
                }
                name_indicate += 1;
            }
        } else {
            break;
        }
    }

    root
}

fn get_path(output: String) -> String {
    let path_start = output.find("Path = ").expect("No path") + 7;
    let path_end = output.find("\nType = ").expect("No path end");

    if let Some(path) = output.get(path_start..path_end) {
        String::from(path)
    } else {
        String::from("Unreacheable")
    }
}

fn main() {
    let args: Vec<String> = args().collect();
    if args.len() != 2 {
        eprintln!("Usage:\n\t{} {{7zip file}}", &args[0]);
        exit(-1);
    }

    let res = Command::new("7z")
        .args(vec!["l", &args[1]])
        .output()
        .expect("Ubo un error al ejecutar el commando!");
    let output = String::from_utf8(res.stdout).expect("No se pudo convertir la salida a texto");
    //println!("El resultado es:\n{}", output);

    let path = get_path(output.clone());
    println!("Path: {}", path);

    let root = get_root(output);
    root.print();
}
