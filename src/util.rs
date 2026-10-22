//! Small helpers shared across the application.

/// Formats a byte count in a human readable way (e.g. `1.4 MiB`).
pub fn human_size(bytes: u64) -> String {
    const UNITS: [&str; 6] = ["B", "KiB", "MiB", "GiB", "TiB", "PiB"];

    if bytes < 1024 {
        return format!("{bytes} B");
    }

    let mut value = bytes as f64;
    let mut unit = 0usize;
    while value >= 1024.0 && unit + 1 < UNITS.len() {
        value /= 1024.0;
        unit += 1;
    }

    if value >= 100.0 {
        format!("{value:.0} {}", UNITS[unit])
    } else if value >= 10.0 {
        format!("{value:.1} {}", UNITS[unit])
    } else {
        format!("{value:.2} {}", UNITS[unit])
    }
}

/// Returns the file name of a path, falling back to the whole path.
pub fn file_name_of(path: &std::path::Path) -> String {
    path.file_name()
        .map(|name| name.to_string_lossy().to_string())
        .unwrap_or_else(|| path.to_string_lossy().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn human_size_scales_units() {
        assert_eq!(human_size(0), "0 B");
        assert_eq!(human_size(512), "512 B");
        assert_eq!(human_size(1024), "1.00 KiB");
        assert_eq!(human_size(1536), "1.50 KiB");
        assert_eq!(human_size(1024 * 1024 * 3 / 2), "1.50 MiB");
    }

    #[test]
    fn file_name_is_taken_from_the_path() {
        assert_eq!(file_name_of(std::path::Path::new("/a/b/c.7z")), "c.7z");
    }
}
