// app/audiolib-desktop/src/reaper_bridge.rs
//! Bounded local file protocol to an explicitly loaded REAPER companion script.
//! This client never starts REAPER, changes a project itself or synthesizes input.
use std::{
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    path::{Path, PathBuf},
    sync::{
        atomic::AtomicU64,
        atomic::{AtomicBool, Ordering},
    },
    thread,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

const HEADER: &str = "ALE_REAPER_1";
const LIMIT: u64 = 16 * 1024;
static SEQUENCE: AtomicU64 = AtomicU64::new(0);

/// Successful acknowledgement, including the exact REAPER window that inserted.
#[derive(Debug)]
pub(crate) struct Delivery {
    pub project: String,
    pub position: Option<f64>,
    pub window: String,
}

/// Transport is chosen before publication, never retried through another path.
pub(crate) enum Destination {
    Companion(Bridge),
    Native(audiolib_windows::NativeReaper),
}

impl Destination {
    pub fn send(&self, path: &Path, cancel: &AtomicBool) -> Result<Delivery, String> {
        match self {
            Self::Companion(bridge) => bridge.send(path, cancel),
            Self::Native(target) => Ok(Delivery {
                project: "REAPER".into(),
                position: None,
                window: target.send(path, cancel)?,
            }),
        }
    }
}

/// Client belonging to one AudioLib data directory; all calls run on workers.
pub(crate) struct Bridge {
    root: PathBuf,
}

fn seconds() -> Result<u64, String> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|n| n.as_secs())
        .map_err(|e| e.to_string())
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

fn reaper_path(path: &str) -> String {
    if let Some(unc) = path.strip_prefix(r"\\?\UNC\") {
        format!(r"\\{unc}")
    } else {
        path.strip_prefix(r"\\?\").unwrap_or(path).to_string()
    }
}

fn unhex(value: &str) -> Result<String, String> {
    if !value.len().is_multiple_of(2) || !value.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err("Malformed REAPER response".into());
    }
    let bytes = (0..value.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&value[i..i + 2], 16).map_err(|e| e.to_string()))
        .collect::<Result<Vec<_>, _>>()?;
    String::from_utf8(bytes).map_err(|e| e.to_string())
}

fn read(path: &Path) -> Result<String, String> {
    let mut bytes = Vec::new();
    File::open(path)
        .map_err(|e| e.to_string())?
        .take(LIMIT + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    if bytes.len() as u64 > LIMIT {
        return Err("REAPER response is too large".into());
    }
    String::from_utf8(bytes).map_err(|e| e.to_string())
}

impl Bridge {
    /// An explicitly running companion remains an opt-in alternative. Otherwise
    /// resolve one native destination before making a durable region export.
    pub fn destination(self) -> Result<Destination, String> {
        if self.connected().is_ok() {
            Ok(Destination::Companion(self))
        } else {
            audiolib_windows::NativeReaper::discover().map(Destination::Native)
        }
    }
    /// Use only the caller's isolated AudioLib data directory.
    pub fn new(data_root: &Path) -> Self {
        Self {
            root: data_root.join("reaper-bridge"),
        }
    }

    /// Write the companion for manual loading through REAPER's Actions window.
    pub fn install(&self) -> Result<PathBuf, String> {
        fs::create_dir_all(&self.root).map_err(|e| e.to_string())?;
        let root = self.root.canonicalize().map_err(|e| e.to_string())?;
        let root = root.to_str().ok_or("The bridge path is not UTF-8")?;
        let script = include_str!("../reaper/AudioLib_Explorer_Bridge.lua")
            .replace("__BRIDGE_ROOT_HEX__", &hex(reaper_path(root).as_bytes()));
        let path = self.root.join("AudioLib_Explorer_Bridge.lua");
        fs::write(&path, script).map_err(|e| e.to_string())?;
        Ok(path)
    }

    /// Reject missing/stale companions before preparing a durable export.
    pub fn connected(&self) -> Result<String, String> {
        let text = read(&self.root.join("session.txt")).map_err(|_| {
            "Connect REAPER first: load and run AudioLib_Explorer_Bridge.lua from Actions"
                .to_string()
        })?;
        let lines: Vec<_> = text.lines().collect();
        if lines.len() != 4
            || lines[0] != HEADER
            || lines[1].len() != 32
            || !lines[1].bytes().all(|b| b.is_ascii_hexdigit())
        {
            return Err("REAPER bridge is not ready".into());
        }
        let stamp = lines[3]
            .parse::<u64>()
            .map_err(|_| "REAPER bridge is not ready")?;
        let now = seconds()?;
        if stamp > now + 1 || now.saturating_sub(stamp) > 3 {
            return Err("REAPER bridge is offline or busy; run the companion in REAPER".into());
        }
        Ok(lines[1].into())
    }

    /// Submit once, never retry an uncertain import. The caller retains its WAV.
    pub fn send(&self, source: &Path, cancel: &AtomicBool) -> Result<Delivery, String> {
        self.send_with_timeout(source, cancel, Duration::from_secs(12))
    }

    fn send_with_timeout(
        &self,
        source: &Path,
        cancel: &AtomicBool,
        timeout: Duration,
    ) -> Result<Delivery, String> {
        let session = self.connected()?;
        let lease = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(self.root.join("send.lock"))
            .map_err(|e| e.to_string())?;
        lease
            .try_lock()
            .map_err(|_| "Another REAPER send is in progress")?;
        if cancel.load(Ordering::Acquire) {
            return Err("REAPER send cancelled".into());
        }
        if !source.is_file() {
            return Err("The original audio file is unavailable".into());
        }
        let path = source.canonicalize().map_err(|e| e.to_string())?;
        let path = path.to_str().ok_or("The audio path is not UTF-8")?;
        let id = format!(
            "{:016x}{:08x}{:08x}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map_err(|e| e.to_string())?
                .as_nanos() as u64,
            std::process::id(),
            SEQUENCE.fetch_add(1, Ordering::Relaxed) as u32
        );
        let pending = self.root.join(format!("{id}.pending"));
        let request = self.root.join("request.txt");
        // A crashed client may leave an unconsumed request. Retire only a known,
        // expired envelope while holding the send lease. Never replay it.
        if let Ok(text) = read(&request) {
            let lines: Vec<_> = text.lines().collect();
            if lines.len() == 5
                && lines[0] == HEADER
                && lines[1].len() == 32
                && lines[1].bytes().all(|b| b.is_ascii_hexdigit())
                && lines[2].len() == 32
                && lines[2].bytes().all(|b| b.is_ascii_hexdigit())
                && lines[3]
                    .parse::<u64>()
                    .is_ok_and(|stamp| seconds().is_ok_and(|now| stamp < now))
            {
                let _ = fs::remove_file(&request);
            }
        }
        if request.exists() {
            return Err(
                "REAPER has an unacknowledged send; wait for it to finish before sending again"
                    .into(),
            );
        }
        let reply = self.root.join(format!("{id}.reply"));
        let mut output = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&pending)
            .map_err(|e| e.to_string())?;
        let payload = format!(
            "{HEADER}\n{session}\n{id}\n{}\n{}\n",
            seconds()? + timeout.as_secs().max(1),
            hex(path.as_bytes())
        );
        if payload.len() as u64 > LIMIT {
            return Err("The audio path exceeds the REAPER bridge limit".into());
        }
        output
            .write_all(payload.as_bytes())
            .map_err(|e| e.to_string())?;
        output.sync_all().map_err(|e| e.to_string())?;
        drop(output);
        fs::rename(&pending, &request).map_err(|e| e.to_string())?;
        let deadline = Instant::now() + timeout;
        loop {
            if let Ok(text) = read(&reply) {
                let lines: Vec<_> = text.lines().collect();
                if lines.len() >= 4 && lines[0] == HEADER && lines[1] == id {
                    let result = if lines[2] == "ERROR" {
                        Err(unhex(lines[3])?)
                    } else if lines.len() == 6 && lines[2] == "OK" {
                        let position = lines[4]
                            .parse::<f64>()
                            .map_err(|_| "Invalid REAPER cursor")?;
                        if !position.is_finite() || position < 0.0 {
                            return Err("Invalid REAPER cursor".into());
                        }
                        Ok(Delivery {
                            project: unhex(lines[3])?,
                            position: Some(position),
                            window: lines[5].into(),
                        })
                    } else {
                        Err("Malformed REAPER acknowledgement".into())
                    };
                    let _ = fs::remove_file(&reply);
                    return result;
                }
            }
            if cancel.load(Ordering::Acquire) || Instant::now() >= deadline {
                // Remove only this request if REAPER has not consumed it. An import
                // already underway may have succeeded, so retain its source and do
                // not automatically submit a duplicate.
                if read(&request)
                    .ok()
                    .is_some_and(|v| v.lines().nth(2) == Some(&id))
                {
                    let _ = fs::remove_file(&request);
                }
                return Err("REAPER did not confirm delivery. Check the project before retrying; the exported WAV was retained".into());
            }
            thread::sleep(Duration::from_millis(15));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn session(bridge: &Bridge) {
        fs::create_dir_all(&bridge.root).unwrap();
        fs::write(
            bridge.root.join("session.txt"),
            format!(
                "{HEADER}\n{}\nuserdata: 123\n{}\n",
                "a".repeat(32),
                seconds().unwrap()
            ),
        )
        .unwrap();
    }
    #[test]
    fn file_peer_acknowledges_unicode_paths_without_launching_any_app() {
        let temp = tempfile::tempdir().unwrap();
        let bridge = Bridge::new(temp.path());
        session(&bridge);
        let source = temp.path().join("seleção '音效'.wav");
        fs::write(&source, b"original PCM").unwrap();
        let root = bridge.root.clone();
        let expected = source.canonicalize().unwrap();
        let peer = thread::spawn(move || {
            let deadline = Instant::now() + Duration::from_secs(3);
            let request = root.join("request.txt");
            while !request.exists() {
                assert!(Instant::now() < deadline);
                thread::sleep(Duration::from_millis(5));
            }
            let text = read(&request).unwrap();
            let lines: Vec<_> = text.lines().collect();
            assert_eq!(unhex(lines[4]).unwrap(), expected.to_str().unwrap());
            fs::remove_file(&request).unwrap();
            fs::write(
                root.join(format!("{}.reply", lines[2])),
                format!(
                    "{HEADER}\n{}\nOK\n{}\n2.5\nuserdata: 123\n",
                    lines[2],
                    hex("Projeto 音效".as_bytes())
                ),
            )
            .unwrap();
        });
        let result = bridge.send(&source, &AtomicBool::new(false)).unwrap();
        peer.join().unwrap();
        assert_eq!(result.project, "Projeto 音效");
        assert_eq!(result.position, Some(2.5));
        assert_eq!(result.window, "userdata: 123");
        assert_eq!(fs::read(source).unwrap(), b"original PCM");
    }
    #[test]
    fn stale_missing_busy_and_cancelled_requests_do_not_enqueue() {
        let temp = tempfile::tempdir().unwrap();
        let bridge = Bridge::new(temp.path());
        assert!(bridge.connected().is_err());
        session(&bridge);
        fs::write(
            bridge.root.join("session.txt"),
            format!("{HEADER}\n{}\nwindow\n0\n", "a".repeat(32)),
        )
        .unwrap();
        assert!(bridge.connected().is_err());
        session(&bridge);
        assert!(
            bridge
                .send(Path::new("missing.wav"), &AtomicBool::new(true))
                .is_err()
        );
        assert!(!bridge.root.join("request.txt").exists());
        let source = temp.path().join("source.wav");
        fs::write(&source, b"PCM").unwrap();
        fs::write(bridge.root.join("request.txt"), b"other request").unwrap();
        assert!(bridge.send(&source, &AtomicBool::new(false)).is_err());
        assert_eq!(
            fs::read(bridge.root.join("request.txt")).unwrap(),
            b"other request"
        );
    }
    #[test]
    fn timeout_retains_original_and_does_not_replay() {
        let temp = tempfile::tempdir().unwrap();
        let bridge = Bridge::new(temp.path());
        session(&bridge);
        let source = temp.path().join("source.wav");
        fs::write(&source, b"PCM").unwrap();
        assert!(
            bridge
                .send_with_timeout(&source, &AtomicBool::new(false), Duration::from_millis(30))
                .is_err()
        );
        assert!(!bridge.root.join("request.txt").exists());
        assert_eq!(fs::read(source).unwrap(), b"PCM");
    }

    #[test]
    fn expired_crash_request_is_not_replayed_and_inflight_cancel_releases_lease() {
        let temp = tempfile::tempdir().unwrap();
        let bridge = Bridge::new(temp.path());
        session(&bridge);
        let source = temp.path().join("source.wav");
        fs::write(&source, b"PCM").unwrap();
        let old_id = "d".repeat(32);
        fs::write(
            bridge.root.join("request.txt"),
            format!(
                "{HEADER}\n{}\n{old_id}\n0\n{}\n",
                "a".repeat(32),
                hex(b"old.wav")
            ),
        )
        .unwrap();
        let cancel = std::sync::Arc::new(AtomicBool::new(false));
        let peer_cancel = cancel.clone();
        let root = bridge.root.clone();
        let peer = thread::spawn(move || {
            let deadline = Instant::now() + Duration::from_secs(3);
            loop {
                if read(&root.join("request.txt"))
                    .ok()
                    .is_some_and(|text| text.lines().nth(2) != Some(&old_id))
                {
                    let other = Bridge { root: root.clone() };
                    assert!(
                        other
                            .send_with_timeout(
                                Path::new("unused.wav"),
                                &AtomicBool::new(false),
                                Duration::from_millis(30)
                            )
                            .unwrap_err()
                            .contains("Another REAPER send")
                    );
                    peer_cancel.store(true, Ordering::Release);
                    return;
                }
                assert!(Instant::now() < deadline);
                thread::sleep(Duration::from_millis(5));
            }
        });
        assert!(bridge.send(&source, &cancel).is_err());
        peer.join().unwrap();
        assert!(!bridge.root.join("request.txt").exists());
        assert_eq!(fs::read(source).unwrap(), b"PCM");
        let lease = OpenOptions::new()
            .read(true)
            .write(true)
            .open(bridge.root.join("send.lock"))
            .unwrap();
        assert!(lease.try_lock().is_ok());
    }
}
