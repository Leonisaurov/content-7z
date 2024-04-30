use crate::files::folder::Folder;

#[derive(Clone, Debug)]
pub enum Entry {
    File(String),
    Folder(Folder),
}

#[derive(Clone, Debug)]
pub enum EntryType {
    File,
    Folder
}
