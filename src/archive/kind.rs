//! Detection of the tool that has to be used for a given archive.

use std::path::Path;

/// The kind of archive, which decides the tool used to list and extract it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArchiveKind {
    /// Anything handled by `7z` (7z, rar, iso, ...).
    SevenZip,
    /// Tarballs, optionally compressed (`.tar`, `.tar.gz`, `.tar.xz`, ...).
    Tar,
    /// Zip archives, handled by `unzip`.
    Zip,
}

impl ArchiveKind {
    /// Guesses the kind of archive from its name.
    pub fn detect(path: &Path) -> Self {
        let name = path
            .file_name()
            .map(|name| name.to_string_lossy().to_lowercase())
            .unwrap_or_default();

        const TAR_SUFFIXES: [&str; 11] = [
            ".tar", ".tar.gz", ".tgz", ".tar.bz2", ".tbz2", ".tbz", ".tar.xz", ".txz", ".tar.zst",
            ".tzst", ".tar.lz4",
        ];

        if TAR_SUFFIXES.iter().any(|suffix| name.ends_with(suffix)) {
            return ArchiveKind::Tar;
        }

        if name.ends_with(".zip") {
            return ArchiveKind::Zip;
        }

        ArchiveKind::SevenZip
    }

    /// The command used to list and extract archives of this kind.
    pub fn tool(&self) -> &'static str {
        match self {
            ArchiveKind::SevenZip => "7z",
            ArchiveKind::Tar => "tar",
            ArchiveKind::Zip => "unzip",
        }
    }

    /// The kinds that can be tried when the preferred tool is not available.
    pub fn fallbacks(&self) -> Vec<ArchiveKind> {
        let mut kinds = vec![*self];
        for kind in [ArchiveKind::SevenZip, ArchiveKind::Tar, ArchiveKind::Zip] {
            if !kinds.contains(&kind) {
                kinds.push(kind);
            }
        }
        kinds
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_tarballs() {
        assert_eq!(
            ArchiveKind::detect(Path::new("archive.tar.gz")),
            ArchiveKind::Tar
        );
        assert_eq!(
            ArchiveKind::detect(Path::new("archive.tgz")),
            ArchiveKind::Tar
        );
        assert_eq!(
            ArchiveKind::detect(Path::new("archive.tar")),
            ArchiveKind::Tar
        );
    }

    #[test]
    fn detects_zips() {
        assert_eq!(
            ArchiveKind::detect(Path::new("archive.zip")),
            ArchiveKind::Zip
        );
    }

    #[test]
    fn defaults_to_seven_zip() {
        assert_eq!(
            ArchiveKind::detect(Path::new("archive.7z")),
            ArchiveKind::SevenZip
        );
        assert_eq!(
            ArchiveKind::detect(Path::new("archive.rar")),
            ArchiveKind::SevenZip
        );
    }

    #[test]
    fn fallbacks_include_the_preferred_kind() {
        let fallbacks = ArchiveKind::Zip.fallbacks();
        assert_eq!(fallbacks.first(), Some(&ArchiveKind::Zip));
        assert!(fallbacks.contains(&ArchiveKind::SevenZip));
    }
}
