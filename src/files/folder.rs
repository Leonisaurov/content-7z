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

    pub fn add_file(&mut self, file_name: &str) {
        if !self.contain_entry(file_name) {
            self.content.push(Entry::File(file_name.to_string()));
        }
    }

    pub fn add_folder(&mut self, folder: Folder) {
        if !self.contain_entry(&folder.name) {
            self.content.push(Entry::Folder(folder));
        }
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

    pub fn contain_entry(&self, entry_name: &str) -> bool {
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

    fn is_safe_component(name: &str) -> bool {
        !name.is_empty() && name != "." && name != ".." && !name.contains('\0')
    }

    pub fn add_entry(&mut self, entry: &str, file_type: &EntryType) {
        // Normalize: strip leading separators and reject traversal.
        let entry = entry.trim_start_matches('/');
        if entry.is_empty() {
            return;
        }
        let parts: Vec<&str> = entry.split('/').filter(|p| !p.is_empty()).collect();
        if parts.iter().any(|p| !Self::is_safe_component(p)) {
            return;
        }

        if parts.len() == 1 {
            if let EntryType::Folder = file_type {
                self.add_folder(Folder::new(parts[0]));
            } else {
                self.add_file(parts[0]);
            }
            return;
        }

        let head = parts[0];
        let rest = parts[1..].join("/");
        if let Some(folder) = self.get_folder(head) {
            folder.add_entry(&rest, file_type);
        } else {
            let mut new_entry = Folder::new(head);
            new_entry.add_entry(&rest, file_type);
            self.add_folder(new_entry);
        }
    }

    pub fn strace(&self, indent: usize) {
        let indent_char = " ".repeat(indent);
        println!("{}Folder: {}", " ".repeat(indent.saturating_sub(1)) + "└┬", self.name);
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
