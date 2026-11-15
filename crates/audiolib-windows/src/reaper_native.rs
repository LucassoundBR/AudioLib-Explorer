// crates/audiolib-windows/src/reaper_native.rs
//! Native REAPER import. No process launch, UI input, polling or automatic retry.
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
#[path = "reaper_protocol.rs"]
#[cfg_attr(not(windows), allow(dead_code))]
mod protocol;

/// Identity of one unambiguous running REAPER main window.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NativeReaper {
    pid: u32,
    started: u64,
    window: usize,
}

impl NativeReaper {
    /// Resolve exactly one verified REAPER; never guess between open instances.
    pub fn discover() -> Result<Self, String> {
        let target = discover()?;
        #[cfg(windows)]
        if !window_ready(target.window) {
            return Err("Finish REAPER startup and close its dialogs before sending".into());
        }
        #[cfg(windows)]
        extension_receiver(target.pid)?;
        Ok(target)
    }

    /// Submit once on a worker. Caller must retain source after uncertain delivery.
    pub fn send(self, source: &Path, cancel: &AtomicBool) -> Result<String, String> {
        if cancel.load(Ordering::Acquire) {
            return Err("REAPER send cancelled".into());
        }
        let path = source.canonicalize().map_err(|e| e.to_string())?;
        if !path.is_file() {
            return Err("The original audio file is unavailable".into());
        }
        let bytes = payload(path.to_str().ok_or("The audio path is not UTF-8")?)?;
        if cancel.load(Ordering::Acquire) {
            return Err("REAPER send cancelled".into());
        }
        let window = native_send(self.pid, self.started, self.window, &bytes)?;
        Ok(format!("userdata: {window:x}"))
    }
}

#[cfg(windows)]
fn window_ready(window: usize) -> bool {
    use std::ffi::c_void;
    #[link(name = "user32")]
    unsafe extern "system" {
        fn IsWindowEnabled(window: *mut c_void) -> i32;
        fn GetLastActivePopup(window: *mut c_void) -> *mut c_void;
        fn IsWindowVisible(window: *mut c_void) -> i32;
    }
    let window = window as *mut c_void;
    // SAFETY: Win32 validates HWNDs; scalar-only queries retain no pointers.
    // Preferences may be modeless, so enabled main HWND alone is insufficient.
    unsafe {
        let popup = GetLastActivePopup(window);
        IsWindowEnabled(window) != 0 && (popup == window || IsWindowVisible(popup) == 0)
    }
}

fn choose(mut targets: Vec<NativeReaper>) -> Result<NativeReaper, String> {
    if targets.iter().any(|target| target.started == 0) {
        return Err(
            "Cannot verify a REAPER window; close other instances or use matching permissions"
                .into(),
        );
    }
    match targets.len() {
        0 => Err("Open REAPER and finish startup before sending; no script is required".into()),
        1 => Ok(targets.remove(0)),
        _ => Err(
            "Multiple REAPER windows are open; keep one destination instance open before sending"
                .into(),
        ),
    }
}

#[cfg(not(windows))]
fn discover() -> Result<NativeReaper, String> {
    choose(Vec::new())
}

#[cfg(windows)]
fn discover() -> Result<NativeReaper, String> {
    use std::ffi::c_void;
    type Handle = *mut c_void;
    #[repr(C)]
    #[derive(Default)]
    struct FileTime {
        low: u32,
        high: u32,
    }
    #[link(name = "user32")]
    unsafe extern "system" {
        fn EnumWindows(
            callback: unsafe extern "system" fn(Handle, isize) -> i32,
            data: isize,
        ) -> i32;
        fn GetWindowThreadProcessId(window: Handle, pid: *mut u32) -> u32;
        fn GetClassNameW(window: Handle, class: *mut u16, size: i32) -> i32;
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
    }
    unsafe extern "system" fn visit(window: Handle, data: isize) -> i32 {
        let mut class = [0_u16; 128];
        // SAFETY: Win32 validates HWND, buffer has the declared writable capacity.
        let length = unsafe { GetClassNameW(window, class.as_mut_ptr(), class.len() as i32) };
        if length <= 0 || class[..length as usize] != "REAPERwnd".encode_utf16().collect::<Vec<_>>()
        {
            return 1;
        }
        let mut pid = 0;
        // SAFETY: Valid scalar output; query-only access to window owner.
        let process = unsafe {
            GetWindowThreadProcessId(window, &mut pid);
            OpenProcess(0x1000, 0, pid)
        };
        if process.is_null() {
            // SAFETY: The enumeration-owned Vec remains valid synchronously.
            unsafe { &mut *(data as *mut Vec<NativeReaper>) }.push(NativeReaper {
                pid,
                started: 0,
                window: window as usize,
            });
            return 1;
        }
        let mut image = [0_u16; 32768];
        let mut length = image.len() as u32;
        let (mut created, mut exit, mut kernel, mut user) = (
            FileTime::default(),
            FileTime::default(),
            FileTime::default(),
            FileTime::default(),
        );
        // SAFETY: Held query handle and valid output buffers; always close once.
        let valid = unsafe {
            let ok = QueryFullProcessImageNameW(process, 0, image.as_mut_ptr(), &mut length) != 0
                && GetProcessTimes(process, &mut created, &mut exit, &mut kernel, &mut user) != 0;
            CloseHandle(process);
            ok
        };
        if !valid || length as usize > image.len() {
            // SAFETY: Do not silently ignore an unverifiable destination window.
            unsafe { &mut *(data as *mut Vec<NativeReaper>) }.push(NativeReaper {
                pid,
                started: 0,
                window: window as usize,
            });
        } else {
            let image = String::from_utf16_lossy(&image[..length as usize]);
            if Path::new(&image)
                .file_name()
                .is_some_and(|s| s.eq_ignore_ascii_case("reaper.exe"))
            {
                // SAFETY: EnumWindows exclusively borrows this Vec synchronously.
                unsafe { &mut *(data as *mut Vec<NativeReaper>) }.push(NativeReaper {
                    pid,
                    started: (u64::from(created.high) << 32) | u64::from(created.low),
                    window: window as usize,
                });
            }
        }
        1
    }
    let mut targets = Vec::<NativeReaper>::new();
    // SAFETY: Callback cannot retain the vector, which outlives enumeration.
    if unsafe { EnumWindows(visit, &mut targets as *mut Vec<NativeReaper> as isize) } == 0 {
        return Err("Cannot enumerate REAPER destination windows".into());
    }
    choose(targets)
}
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
    let mut bytes = path.into_bytes();
    bytes.push(0);
    if bytes.len() > 32 * 1024 {
        return Err("The audio path is too long for native REAPER import".into());
    }
    Ok(bytes)
}

#[cfg(any(windows, test))]
fn outcome(transport: bool, receiver: usize, error: u32) -> Result<&'static str, String> {
    if !transport {
        Err(format!(
            "Delivery unconfirmed (Win32 {error}); inspect REAPER before any manual retry"
        ))
    } else {
        match receiver {
            protocol::ACCEPTED => {
                Ok("REAPER extension accepted insertion; verify actual insertion")
            }
            protocol::NO_TRACK => Err("Select exactly one destination track in REAPER".into()),
            protocol::BUSY => {
                Err("REAPER extension is busy; inspect REAPER before retrying".into())
            }
            protocol::REJECTED => {
                Err("REAPER extension rejected the request before insertion".into())
            }
            protocol::UNCONFIRMED => {
                Err("Insertion unconfirmed; inspect REAPER before any manual retry".into())
            }
            _ => Err("Insertion unconfirmed; inspect REAPER before any manual retry".into()),
        }
    }
}

#[cfg(windows)]
fn extension_receiver(pid: u32) -> Result<usize, String> {
    use std::ffi::c_void;
    type Hwnd = *mut c_void;
    struct Search {
        pid: u32,
        windows: Vec<usize>,
    }
    #[link(name = "user32")]
    unsafe extern "system" {
        fn EnumWindows(callback: unsafe extern "system" fn(Hwnd, isize) -> i32, data: isize)
        -> i32;
        fn GetWindowThreadProcessId(window: Hwnd, pid: *mut u32) -> u32;
        fn GetClassNameW(window: Hwnd, class: *mut u16, size: i32) -> i32;
        fn SendMessageTimeoutW(
            window: Hwnd,
            msg: u32,
            w: usize,
            l: isize,
            flags: u32,
            timeout: u32,
            result: *mut usize,
        ) -> isize;
    }
    unsafe extern "system" fn visit(window: Hwnd, data: isize) -> i32 {
        // SAFETY: Synchronous enumeration borrows Search; query-only HWND calls.
        let search = unsafe { &mut *(data as *mut Search) };
        let mut owner = 0;
        let mut class = [0u16; 128];
        // SAFETY: Scalar output and bounded writable class-name buffer.
        let length = unsafe {
            GetWindowThreadProcessId(window, &mut owner);
            GetClassNameW(window, class.as_mut_ptr(), 128)
        };
        if owner == search.pid
            && length > 0
            && class[..length as usize] == protocol::CLASS_NAME.encode_utf16().collect::<Vec<_>>()
        {
            search.windows.push(window as usize);
        }
        1
    }
    let mut search = Search {
        pid,
        windows: Vec::new(),
    };
    // SAFETY: Search remains alive for this synchronous enumeration only.
    if unsafe { EnumWindows(visit, &mut search as *mut Search as isize) } == 0
        || search.windows.len() != 1
    {
        return Err(
            "Install the AudioLib native DLL via Connect REAPER, then restart REAPER".into(),
        );
    }
    let receiver = search.windows[0];
    let mut version = 0;
    // SAFETY: Bounded scalar handshake to the extension in the verified host PID.
    if unsafe {
        SendMessageTimeoutW(
            receiver as Hwnd,
            protocol::HELLO,
            0,
            0,
            0x23,
            1000,
            &mut version,
        )
    } == 0
        || version != protocol::VERSION
    {
        return Err(
            "REAPER extension is unavailable or incompatible; update the DLL and restart REAPER"
                .into(),
        );
    }
    let mut ready = 0;
    // SAFETY: Readiness is query-only, with the same bounded scalar transport.
    if unsafe {
        SendMessageTimeoutW(
            receiver as Hwnd,
            protocol::READY,
            0,
            0,
            0x23,
            1000,
            &mut ready,
        )
    } == 0
    {
        return Err("REAPER extension did not respond; no audio was sent".into());
    }
    outcome(true, ready, 0)?;
    Ok(receiver)
}

#[cfg(not(windows))]
fn native_send(_: u32, _: u64, _: usize, _: &[u8]) -> Result<usize, String> {
    Err("This experiment requires Windows".into())
}

#[cfg(windows)]
fn native_send(
    pid: u32,
    started: u64,
    expected_window: usize,
    bytes: &[u8],
) -> Result<usize, String> {
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
    if window as usize != expected_window {
        return Err("REAPER window changed; select the destination again".into());
    }
    let mut owner = 0;
    let mut exit_code = 0;
    // SAFETY: Recheck ownership/liveness immediately before submitting once.
    let ready = unsafe {
        GetWindowThreadProcessId(window, &mut owner);
        GetExitCodeProcess(process.0, &mut exit_code) != 0
            && exit_code == 259
            && owner == pid
            && window_ready(window as usize)
    };
    if !ready {
        return Err(
            "REAPER closed or has a dialog open; finish startup and close dialogs before sending"
                .into(),
        );
    }
    let receiver_window = extension_receiver(pid)?;
    let packet = CopyData {
        kind: protocol::PACKET,
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
            receiver_window as Handle,
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
    outcome(sent != 0, receiver, error)?;
    Ok(window as usize)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn native_payload_preserves_literal_unicode_and_normalizes_prefixes() {
        let path = r"C:\seleção '音效' #1.wav";
        assert_eq!(payload(path).unwrap(), format!("{path}\0").into_bytes());
        assert_eq!(
            payload(r"\\?\C:\test.wav").unwrap(),
            payload(r"C:\test.wav").unwrap()
        );
        assert_eq!(
            payload(r"\\?\UNC\nas\sfx\test.wav").unwrap(),
            payload(r"\\nas\sfx\test.wav").unwrap()
        );
        for bad in [
            "x.wav",
            r"C:x.wav",
            r"\\.\pipe\x",
            "C:\\x\0.wav",
            "C:\\x\n.wav",
        ] {
            assert!(payload(bad).is_err());
        }
        assert!(payload(&format!("C:\\{}", "x".repeat(32768))).is_err());
    }
    #[test]
    fn native_destination_is_never_chosen_arbitrarily() {
        let t = NativeReaper {
            pid: 1,
            started: 2,
            window: 3,
        };
        assert!(choose(vec![]).is_err());
        assert_eq!(choose(vec![t]).unwrap(), t);
        assert!(choose(vec![t, t]).is_err());
        assert!(choose(vec![t, NativeReaper { started: 0, ..t }]).is_err());
    }
    #[test]
    fn native_outcome_and_close_intent_do_not_claim_insertion() {
        assert!(outcome(false, 1, 1460).unwrap_err().contains("unconfirmed"));
        assert!(outcome(true, 0, 0).is_err());
        assert!(outcome(true, protocol::REJECTED, 0).is_err());
        assert!(
            outcome(true, protocol::NO_TRACK, 0)
                .unwrap_err()
                .contains("exactly one")
        );
        assert!(outcome(true, protocol::UNCONFIRMED, 0).is_err());
        assert!(
            outcome(true, 1, 0)
                .unwrap()
                .contains("verify actual insertion")
        );
        let t = NativeReaper {
            pid: 1,
            started: 2,
            window: 3,
        };
        assert!(
            t.send(Path::new("missing.wav"), &AtomicBool::new(true))
                .unwrap_err()
                .contains("cancelled")
        );
    }
}
