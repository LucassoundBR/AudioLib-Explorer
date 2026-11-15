// tools/reaper-native-probe.rs
//! Standalone Windows transport experiment. Default invocation never calls Win32.
//! Not linked into the application; native sends need an authorized idle session.
use std::path::Path;

fn payload(path: &str) -> Result<Vec<u8>, String> {
    let path = if let Some(unc) = path.strip_prefix(r"\\?\UNC\") {
        format!(r"\\{unc}")
    } else {
        path.strip_prefix(r"\\?\").unwrap_or(path).to_owned()
    };
    let drive_absolute = path.as_bytes().get(1) == Some(&b':')
        && path.as_bytes().first().is_some_and(u8::is_ascii_alphabetic)
        && path
            .as_bytes()
            .get(2)
            .is_some_and(|b| *b == b'\\' || *b == b'/');
    let unc_absolute = path.strip_prefix(r"\\").is_some_and(|tail| {
        let parts: Vec<_> = tail.split('\\').collect();
        parts.len() >= 3 && parts.iter().all(|p| !p.is_empty())
    });
    if (!drive_absolute && !unc_absolute)
        || path.starts_with(r"\\.\")
        || path.starts_with(r"\\?\")
        || path.chars().any(char::is_control)
    {
        return Err("Expected an absolute filesystem path without control characters".into());
    }
    let mut bytes = format!("#NOTAB#{path}").into_bytes();
    bytes.push(0);
    if bytes.len() > 32 * 1024 {
        return Err("Native REAPER request exceeds the probe's 32 KiB limit".into());
    }
    Ok(bytes)
}

#[cfg(any(windows, test))]
fn outcome(transport: bool, receiver: usize, error: u32) -> Result<&'static str, String> {
    if !transport {
        Err(format!(
            "Delivery unconfirmed (Win32 {error}); inspect REAPER before any manual retry"
        ))
    } else if receiver == 0 {
        Err(
            "Receiver returned zero; insertion not confirmed. Inspect REAPER; do not auto-retry"
                .into(),
        )
    } else {
        Ok("Receiver returned nonzero; inspect the project to verify actual insertion")
    }
}

fn run(args: &[String]) -> Result<&'static str, String> {
    if args.is_empty() || args == ["--help"] {
        return Ok(
            "Inert by default. Authorized native test only: --interactive PID CREATION_TICKS ABSOLUTE_WAV_PATH. Does not launch or focus REAPER; does not retry.",
        );
    }
    if args.len() != 4 || args[0] != "--interactive" {
        return Err("Expected --interactive PID CREATION_TICKS ABSOLUTE_WAV_PATH".into());
    }
    let pid = args[1].parse::<u32>().map_err(|e| e.to_string())?;
    let started = args[2].parse::<u64>().map_err(|e| e.to_string())?;
    if pid == 0 || started == 0 {
        return Err("An explicit owned process identity is required".into());
    }
    let source = Path::new(&args[3])
        .canonicalize()
        .map_err(|e| e.to_string())?;
    if !source.is_file()
        || !source
            .extension()
            .is_some_and(|s| s.eq_ignore_ascii_case("wav"))
    {
        return Err("The probe requires an existing synthetic WAV".into());
    }
    let bytes = payload(source.to_str().ok_or("Path is not UTF-8")?)?;
    native_send(pid, started, &bytes)
}

#[cfg(not(windows))]
fn native_send(_: u32, _: u64, _: &[u8]) -> Result<&'static str, String> {
    Err("This experiment requires Windows".into())
}

#[cfg(windows)]
fn native_send(pid: u32, started: u64, bytes: &[u8]) -> Result<&'static str, String> {
    use std::{ffi::c_void, ptr};
    type Handle = *mut c_void;
    #[repr(C)]
    #[derive(Default)]
    struct FileTime {
        low: u32,
        high: u32,
    }
    #[repr(C)]
    struct CopyData {
        kind: usize,
        length: u32,
        data: *const u8,
    }
    struct Search {
        pid: u32,
        windows: Vec<Handle>,
    }
    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn OpenProcess(access: u32, inherit: i32, pid: u32) -> Handle;
        fn CloseHandle(handle: Handle) -> i32;
        fn QueryFullProcessImageNameW(
            handle: Handle,
            flags: u32,
            path: *mut u16,
            size: *mut u32,
        ) -> i32;
        fn GetProcessTimes(
            handle: Handle,
            created: *mut FileTime,
            exit: *mut FileTime,
            kernel: *mut FileTime,
            user: *mut FileTime,
        ) -> i32;
        fn GetExitCodeProcess(handle: Handle, code: *mut u32) -> i32;
        fn SetLastError(error: u32);
        fn GetLastError() -> u32;
    }
    #[link(name = "user32")]
    unsafe extern "system" {
        fn EnumWindows(
            callback: unsafe extern "system" fn(Handle, isize) -> i32,
            data: isize,
        ) -> i32;
        fn GetWindowThreadProcessId(window: Handle, pid: *mut u32) -> u32;
        fn GetClassNameW(window: Handle, class: *mut u16, size: i32) -> i32;
        fn IsWindowEnabled(window: Handle) -> i32;
        fn GetLastActivePopup(window: Handle) -> Handle;
        fn IsWindowVisible(window: Handle) -> i32;
        fn SendMessageTimeoutW(
            window: Handle,
            message: u32,
            sender: usize,
            data: isize,
            flags: u32,
            timeout: u32,
            result: *mut usize,
        ) -> isize;
    }
    unsafe extern "system" fn visit(window: Handle, data: isize) -> i32 {
        // SAFETY: EnumWindows synchronously borrows Search for this enumeration.
        let search = unsafe { &mut *(data as *mut Search) };
        let mut owner = 0;
        let mut class = [0_u16; 128];
        // SAFETY: Win32 validates HWND; writable buffers match advertised sizes.
        let length = unsafe {
            GetWindowThreadProcessId(window, &mut owner);
            GetClassNameW(window, class.as_mut_ptr(), class.len() as i32)
        };
        if owner == search.pid
            && length > 0
            && class[..length as usize] == "REAPERwnd".encode_utf16().collect::<Vec<_>>()
        {
            search.windows.push(window);
        }
        1
    }
    struct Process(Handle);
    impl Drop for Process {
        fn drop(&mut self) {
            // SAFETY: This guard owns the non-null OpenProcess handle exactly once.
            unsafe {
                CloseHandle(self.0);
            }
        }
    }
    // SAFETY: Query-only access; no process state is changed or terminated.
    let handle = unsafe { OpenProcess(0x1000, 0, pid) };
    if handle.is_null() {
        return Err("Cannot query the specified REAPER process".into());
    }
    let process = Process(handle);
    let (mut created, mut exited, mut kernel, mut user) = (
        FileTime::default(),
        FileTime::default(),
        FileTime::default(),
        FileTime::default(),
    );
    let mut executable = [0_u16; 32768];
    let mut length = executable.len() as u32;
    // SAFETY: Held process handle and valid writable buffers; no retained pointers.
    let valid = unsafe {
        GetProcessTimes(process.0, &mut created, &mut exited, &mut kernel, &mut user) != 0
            && QueryFullProcessImageNameW(process.0, 0, executable.as_mut_ptr(), &mut length) != 0
    };
    if !valid || length as usize > executable.len() {
        return Err("Cannot read the owned process identity".into());
    }
    let ticks = (u64::from(created.high) << 32) | u64::from(created.low);
    let executable = String::from_utf16_lossy(&executable[..length as usize]);
    if ticks != started
        || !Path::new(&executable)
            .file_name()
            .is_some_and(|s| s.eq_ignore_ascii_case("reaper.exe"))
    {
        return Err(
            "REAPER PID, creation time or executable identity differs from the owned run".into(),
        );
    }
    let mut search = Search {
        pid,
        windows: Vec::new(),
    };
    // SAFETY: Search outlives synchronous enumeration; callback never retains it.
    if unsafe { EnumWindows(visit, &mut search as *mut Search as isize) } == 0
        || search.windows.len() != 1
    {
        return Err("Expected exactly one REAPER main window for the explicit PID".into());
    }
    let window = search.windows[0];
    let mut owner = 0;
    let mut exit_code = 0;
    // SAFETY: Recheck ownership/liveness immediately before submitting once.
    let ready = unsafe {
        GetWindowThreadProcessId(window, &mut owner);
        GetExitCodeProcess(process.0, &mut exit_code) != 0
            && exit_code == 259
            && owner == pid
            && IsWindowEnabled(window) != 0
            && (GetLastActivePopup(window) == window
                || IsWindowVisible(GetLastActivePopup(window)) == 0)
    };
    if !ready {
        return Err("Owned REAPER exited or its main window is blocked by a dialog".into());
    }
    let packet = CopyData {
        kind: 0x100,
        length: bytes.len() as u32,
        data: bytes.as_ptr(),
    };
    let mut receiver = 0;
    // SAFETY: System-defined WM_COPYDATA is marshalled across processes. Packet
    // and immutable NUL-terminated payload outlive the synchronous bounded call.
    // No broadcast, focus, input injection or async borrowed pointers are used.
    let sent = unsafe {
        SetLastError(0);
        SendMessageTimeoutW(
            window,
            0x4a,
            0,
            ptr::from_ref(&packet) as isize,
            0x23,
            2000,
            &mut receiver,
        )
    };
    // SAFETY: Reads this thread's last-error value immediately after the send.
    let error = unsafe { GetLastError() };
    outcome(sent != 0, receiver, error)
}

fn main() {
    match run(&std::env::args().skip(1).collect::<Vec<_>>()) {
        Ok(status) => println!("{status}"),
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn preserves_unicode_quotes_hashes_and_spaces_as_literal_utf8() {
        let p = r"C:\test\seleção '音效' #1.wav";
        assert_eq!(payload(p).unwrap(), format!("#NOTAB#{p}\0").into_bytes());
    }
    #[test]
    fn normalizes_extended_drive_and_unc_without_shell_quoting() {
        assert_eq!(
            payload(r"\\?\C:\test.wav").unwrap(),
            payload(r"C:\test.wav").unwrap()
        );
        assert_eq!(
            payload(r"\\?\UNC\nas\sfx\test.wav").unwrap(),
            payload(r"\\nas\sfx\test.wav").unwrap()
        );
    }
    #[test]
    fn rejects_relative_device_control_and_oversized_paths() {
        for p in [
            "",
            "test.wav",
            r"C:test.wav",
            r"\test.wav",
            r"\\.\pipe\test",
            "C:\\a\0.wav",
            "C:\\a\n.wav",
            r"\\nas\share",
        ] {
            assert!(payload(p).is_err(), "{p:?}");
        }
        assert!(payload(&format!("C:\\{}.wav", "x".repeat(32768))).is_err());
    }
    #[test]
    fn timeout_and_zero_receiver_never_claim_success() {
        assert!(outcome(false, 1, 1460).unwrap_err().contains("unconfirmed"));
        assert!(outcome(false, 0, 5).unwrap_err().contains("Win32 5"));
        assert!(outcome(true, 0, 0).is_err());
        assert!(
            outcome(true, 1, 0)
                .unwrap()
                .contains("verify actual insertion")
        );
    }
    #[test]
    fn default_and_invalid_cli_never_reach_native_dispatch() {
        assert!(run(&[]).unwrap().contains("Inert"));
        assert!(run(&["--send".into()]).is_err());
        assert!(
            run(&[
                "--interactive".into(),
                "0".into(),
                "1".into(),
                "C:\\test.wav".into()
            ])
            .is_err()
        );
    }
}
