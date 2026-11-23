// crates/audiolib-windows/src/reaper_extension.rs
//! Export the bundled native REAPER extension without changing a REAPER profile.
use std::path::{Path, PathBuf};

/// Write a standalone DLL for explicit installation in REAPER's UserPlugins.
/// This never installs into, configures, starts or activates the host.
pub fn export_reaper_extension(directory: &Path) -> Result<PathBuf, String> {
    #[cfg(windows)]
    {
        std::fs::create_dir_all(directory).map_err(|e| e.to_string())?;
        let root = directory.canonicalize().map_err(|e| e.to_string())?;
        let target = root.join("reaper_audiolib_bridge.dll");
        let bytes = include_bytes!(concat!(env!("OUT_DIR"), "/reaper_audiolib_bridge.dll"));
        // Persist through a new owned temporary file; never follow an existing
        // destination link or truncate a DLL currently loaded by a host.
        let mut temporary = tempfile::NamedTempFile::new_in(&root).map_err(|e| e.to_string())?;
        std::io::Write::write_all(&mut temporary, bytes).map_err(|e| e.to_string())?;
        temporary.as_file().sync_all().map_err(|e| e.to_string())?;
        temporary.persist(&target).map_err(|e| e.to_string())?;
        Ok(target)
    }
    #[cfg(not(windows))]
    {
        let _ = directory;
        Err("The native REAPER extension requires Windows x64".into())
    }
}

#[cfg(all(test, windows))]
mod tests {
    use super::*;
    #[test]
    fn reaper_export_replaces_only_its_own_dll() {
        let directory = tempfile::tempdir().unwrap();
        let sentinel = directory.path().join("settings.json");
        std::fs::write(&sentinel, b"preserve").unwrap();
        let path = export_reaper_extension(directory.path()).unwrap();
        let original = std::fs::read(&path).unwrap();
        assert!(original.starts_with(b"MZ"));
        assert_eq!(export_reaper_extension(directory.path()).unwrap(), path);
        assert_eq!(std::fs::read(path).unwrap(), original);
        assert_eq!(std::fs::read(sentinel).unwrap(), b"preserve");
    }
    #[test]
    fn reaper_export_preserves_a_locked_previous_dll() {
        use std::os::windows::fs::OpenOptionsExt;
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("reaper_audiolib_bridge.dll");
        std::fs::write(&path, b"previous").unwrap();
        let locked = std::fs::OpenOptions::new()
            .read(true)
            .share_mode(0)
            .open(&path)
            .unwrap();
        assert!(export_reaper_extension(directory.path()).is_err());
        drop(locked);
        assert_eq!(std::fs::read(path).unwrap(), b"previous");
        assert_eq!(std::fs::read_dir(directory.path()).unwrap().count(), 1);
    }
}
