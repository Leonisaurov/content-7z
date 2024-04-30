use crate::files::entry::{Entry, EntryType};

#[derive(Clone, Debug)]
pub struct Folder {
    pub name: String,
    pub content: Vec<Entry>,
}

impl Folder {
    pub fn new(name: &str) -> Self {
        Self {
            name: String::from(name),
            content: Vec::new()
        }
    }

    pub fn get_root(output: String) -> Self {
        let mut root = Self::new(".");

        let start_point: usize = output.find("   Date      Time    Attr         Size   Compressed  Name\n------------------- ----- ------------ ------------  ------------------------\n").expect("The content isn't be found") + "   Date      Time    Attr         Size   Compressed  Name\n------------------- ----- ------------ ------------  ------------------------\n".len();
        let clean_output = &output[start_point..];
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
            //eprintln!("Name: {}, type: {}", &line[53..], &line[20..25]);
        }

        root
    }

    pub fn add_file(&mut self, file_name: &str) {
        self.content.push(Entry::File(file_name.to_string()));
    }

    pub fn add_folder(&mut self, folder: Folder) {
        self.content.push(Entry::Folder(folder));
    }

    pub fn get_folder(&mut self, folder_name: &str) -> Option<&mut Folder> {
        for entry in &mut self.content {
            if let Entry::Folder(folder) = entry {
                if folder.name == folder_name {
                    return Some(folder)
                }
            }
        }
        None
    }

    pub fn contain_entry(&mut self, entry_name: &str) -> bool {
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

    pub fn add_entry(&mut self, entry: &str, file_type: &EntryType) {
        // eprintln!("{}", entry);
        for i in 0..entry.len() {
            if let Some(character) = entry.get(i..i+1) {
                if character == "/" {
                    if let Some(path) = entry.get(0..i) {
                        if self.contain_entry(path) {
                            if let Some(sub_path) = entry.get(i+1..entry.len()) {
                                if let Some(folder) = self.get_folder(path) {
                                    folder.add_entry(sub_path, file_type);
                                } else {
                                    let mut new_entry = Folder::new(path);
                                    // eprintln!("No folder found: {} / {}", path, sub_path);
                                    new_entry.add_entry(sub_path, file_type);
                                    self.add_folder(new_entry);
                                }
                            } else {
                                eprintln!("The subdir cannot be get");
                            }
                            // eprintln!("In the Main Folder {}.", path);
                            return;
                        }
                        let mut new_entry = Folder::new(path);
                        // eprintln!("Main Folder {} Added.", path);
                        if let Some(sub_path) = entry.get(i+1..entry.len()) {
                            new_entry.add_entry(sub_path, file_type);
                            //eprintln!("Se pudo?");
                        }
                        // eprintln!("In the Main Folder {}.", path);
                        self.add_folder(new_entry);

                    } else {
                        eprintln!("The subdir cannot be get");
                    }
                    return;
                }
            }
        }

        if let Some(path) = entry.get(0..entry.len()) {
            if let EntryType::Folder = file_type {
                // eprintln!("Main Folder {} Added withouth follow.", path);
                self.add_folder(Folder::new(path))
            } else {
                // eprintln!("File {} added.", path);
                self.add_file(path);
            }
        } else {
            eprintln!("No se pudo?");
        }

    }


    pub fn strace(&self, indent: usize) {
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

    pub fn print(&self) {
        self.strace(1);
    }
}
