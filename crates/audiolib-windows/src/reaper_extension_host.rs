// crates/audiolib-windows/src/reaper_extension_host.rs
//! Native REAPER extension. All host calls execute on its owning UI thread.
//! ABI reference: https://github.com/justinfrankel/reaper-sdk/blob/main/sdk/reaper_plugin.h
#![cfg(windows)]
#[path = "reaper_protocol.rs"]
mod protocol;
use std::{
    ffi::{CStr, c_char, c_void},
    ptr,
    sync::{
        OnceLock,
        atomic::{AtomicBool, Ordering},
    },
};
type Handle = *mut c_void;
#[repr(C)]
/// Public REAPER extension ABI layout; fields are supplied exclusively by the host.
pub struct PluginInfo {
    version: i32,
    main: Handle,
    register: unsafe extern "C" fn(*const c_char, Handle) -> i32,
    get_func: unsafe extern "C" fn(*const c_char) -> Handle,
}
#[derive(Clone, Copy)]
struct Api {
    main: usize,
    insert: unsafe extern "C" fn(*const c_char, i32) -> i32,
    begin: unsafe extern "C" fn(Handle),
    end: unsafe extern "C" fn(Handle, *const c_char, i32),
    selected: unsafe extern "C" fn(Handle) -> i32,
    project: unsafe extern "C" fn(i32, *mut c_char, i32) -> Handle,
    count: unsafe extern "C" fn(Handle) -> i32,
    cursor: unsafe extern "C" fn(Handle) -> f64,
    set_cursor: unsafe extern "C" fn(Handle, f64, bool, bool),
}
static API: OnceLock<Api> = OnceLock::new();
static BUSY: AtomicBool = AtomicBool::new(false);
static WINDOW: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
static CLASS: OnceLock<Vec<u16>> = OnceLock::new();
fn class_name() -> *const u16 {
    CLASS
        .get_or_init(|| protocol::CLASS_NAME.encode_utf16().chain(Some(0)).collect())
        .as_ptr()
}
#[repr(C)]
struct WindowClass {
    style: u32,
    proc: unsafe extern "system" fn(Handle, u32, usize, isize) -> isize,
    extra: i32,
    window_extra: i32,
    instance: Handle,
    icon: Handle,
    cursor: Handle,
    background: Handle,
    menu: *const u16,
    name: *const u16,
}
#[repr(C)]
struct CopyData {
    kind: usize,
    length: u32,
    data: *const u8,
}
#[link(name = "user32")]
unsafe extern "system" {
    fn RegisterClassW(class: *const WindowClass) -> u16;
    fn UnregisterClassW(name: *const u16, instance: Handle) -> i32;
    fn CreateWindowExW(
        ex: u32,
        class: *const u16,
        title: *const u16,
        style: u32,
        x: i32,
        y: i32,
        w: i32,
        h: i32,
        parent: Handle,
        menu: Handle,
        instance: Handle,
        param: Handle,
    ) -> Handle;
    fn DestroyWindow(window: Handle) -> i32;
    fn DefWindowProcW(window: Handle, msg: u32, w: usize, l: isize) -> isize;
    fn IsWindowEnabled(window: Handle) -> i32;
    fn GetLastActivePopup(window: Handle) -> Handle;
    fn IsWindowVisible(window: Handle) -> i32;
}
fn valid_path(bytes: &[u8]) -> Option<&CStr> {
    let path = CStr::from_bytes_with_nul(bytes).ok()?;
    let text = path.to_str().ok()?;
    if text.chars().any(char::is_control)
        || !std::path::Path::new(text).is_absolute()
        || !std::path::Path::new(text)
            .extension()
            .is_some_and(|s| s.eq_ignore_ascii_case("wav"))
        || !std::path::Path::new(text).is_file()
    {
        return None;
    }
    Some(path)
}
fn readiness(api: Api) -> usize {
    if BUSY.load(Ordering::Acquire) {
        return protocol::BUSY;
    }
    // SAFETY: Only called from this window's owning REAPER main thread; API
    // functions and HWND were obtained from this host and remain valid until unload.
    unsafe {
        let main = api.main as Handle;
        let popup = GetLastActivePopup(main);
        if IsWindowEnabled(main) == 0 || (popup != main && IsWindowVisible(popup) != 0) {
            return protocol::REJECTED;
        }
        let project = (api.project)(-1, ptr::null_mut(), 0);
        if project.is_null() || (api.selected)(project) != 1 {
            protocol::NO_TRACK
        } else {
            protocol::ACCEPTED
        }
    }
}
unsafe fn insert_media(api: Api, path: &CStr) -> usize {
    // SAFETY: Caller runs on the host UI thread with resolved exact SDK function
    // signatures. The current project stays borrowed through this synchronous
    // block; no pointers or C strings are retained by our extension.
    unsafe {
        let project = (api.project)(-1, ptr::null_mut(), 0);
        if project.is_null() || (api.selected)(project) != 1 {
            return protocol::NO_TRACK;
        }
        let before = (api.count)(project);
        let cursor = (api.cursor)(project);
        (api.begin)(project);
        (api.insert)(path.as_ptr(), 0);
        (api.set_cursor)(project, cursor, false, false);
        (api.end)(project, c"AudioLib: send audio".as_ptr(), -1);
        if (api.count)(project) == before + 1 {
            protocol::ACCEPTED
        } else {
            protocol::UNCONFIRMED
        }
    }
}
unsafe extern "system" fn receive(window: Handle, msg: u32, w: usize, l: isize) -> isize {
    if msg == protocol::HELLO {
        return protocol::VERSION as isize;
    }
    if msg == protocol::READY {
        return API
            .get()
            .copied()
            .map(readiness)
            .unwrap_or(protocol::REJECTED) as isize;
    }
    if msg != 0x4a {
        // SAFETY: Forward untouched messages on this registered window.
        return unsafe { DefWindowProcW(window, msg, w, l) };
    }
    if l == 0 {
        return protocol::REJECTED as isize;
    }
    // SAFETY: Windows marshals WM_COPYDATA and holds the packet during this call.
    let packet = unsafe { &*(l as *const CopyData) };
    if packet.kind != protocol::PACKET
        || packet.data.is_null()
        || !(2..=32768).contains(&packet.length)
    {
        return protocol::REJECTED as isize;
    }
    // SAFETY: The marshalled packet advertises this byte count; borrowed only synchronously.
    let bytes = unsafe { std::slice::from_raw_parts(packet.data, packet.length as usize) };
    let Some(path) = valid_path(bytes) else {
        return protocol::REJECTED as isize;
    };
    let Some(api) = API.get().copied() else {
        return protocol::REJECTED as isize;
    };
    let ready = readiness(api);
    if ready != protocol::ACCEPTED {
        return ready as isize;
    }
    if BUSY.swap(true, Ordering::AcqRel) {
        return protocol::BUSY as isize;
    }
    // SAFETY: REAPER creates/delivers this window on its main thread. API pointers
    // are resolved from this host's exact public ABI, and stay valid until unload.
    let result = unsafe {
        let main = api.main as Handle;
        let popup = GetLastActivePopup(main);
        if IsWindowEnabled(main) == 0 || (popup != main && IsWindowVisible(popup) != 0) {
            protocol::REJECTED as isize
        } else {
            insert_media(api, path) as isize
        }
    };
    BUSY.store(false, Ordering::Release);
    result
}

/// REAPER extension entry point; validates the public SDK ABI before activation.
/// # Safety
/// `info` must be the REAPER SDK structure supplied by the host, or null on unload.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ReaperPluginEntry(instance: Handle, info: *const PluginInfo) -> i32 {
    if info.is_null() {
        let window = WINDOW.swap(0, Ordering::AcqRel) as Handle;
        // SAFETY: Host invokes unload on the owning main thread; class belongs to this DLL.
        unsafe {
            if !window.is_null() {
                DestroyWindow(window);
            }
            UnregisterClassW(class_name(), instance);
        }
        return 0;
    }
    // SAFETY: Entry point host owns a valid PluginInfo for this synchronous initialization.
    let info = unsafe { &*info };
    if info.version != 0x20e || info.main.is_null() {
        return 0;
    }
    let names = [
        c"InsertMedia",
        c"Undo_BeginBlock2",
        c"Undo_EndBlock2",
        c"CountSelectedTracks",
        c"EnumProjects",
        c"CountMediaItems",
        c"GetCursorPositionEx",
        c"SetEditCurPos2",
    ];
    let mut funcs = [ptr::null_mut(); 8];
    for (i, name) in names.iter().enumerate() {
        // SAFETY: Host-provided GetFunc accepts static NUL-terminated API names.
        funcs[i] = unsafe { (info.get_func)(name.as_ptr()) };
        if funcs[i].is_null() {
            return 0;
        }
    }
    // SAFETY: Each non-null function has the exact public SDK signature named above.
    let api = unsafe {
        Api {
            main: info.main as usize,
            insert: std::mem::transmute::<Handle, unsafe extern "C" fn(*const c_char, i32) -> i32>(
                funcs[0],
            ),
            begin: std::mem::transmute::<Handle, unsafe extern "C" fn(Handle)>(funcs[1]),
            end: std::mem::transmute::<Handle, unsafe extern "C" fn(Handle, *const c_char, i32)>(
                funcs[2],
            ),
            selected: std::mem::transmute::<Handle, unsafe extern "C" fn(Handle) -> i32>(funcs[3]),
            project: std::mem::transmute::<
                Handle,
                unsafe extern "C" fn(i32, *mut c_char, i32) -> Handle,
            >(funcs[4]),
            count: std::mem::transmute::<Handle, unsafe extern "C" fn(Handle) -> i32>(funcs[5]),
            cursor: std::mem::transmute::<Handle, unsafe extern "C" fn(Handle) -> f64>(funcs[6]),
            set_cursor: std::mem::transmute::<Handle, unsafe extern "C" fn(Handle, f64, bool, bool)>(
                funcs[7],
            ),
        }
    };
    if API.set(api).is_err() {
        return 0;
    }
    let class = WindowClass {
        style: 0,
        proc: receive,
        extra: 0,
        window_extra: 0,
        instance,
        icon: ptr::null_mut(),
        cursor: ptr::null_mut(),
        background: ptr::null_mut(),
        menu: ptr::null(),
        name: class_name(),
    };
    // SAFETY: Static class name/procedure outlive the hidden window; never shown or activated.
    let window = unsafe {
        if RegisterClassW(&class) == 0 {
            return 0;
        }
        CreateWindowExW(
            0,
            class_name(),
            class_name(),
            0,
            0,
            0,
            0,
            0,
            ptr::null_mut(),
            ptr::null_mut(),
            instance,
            ptr::null_mut(),
        )
    };
    if window.is_null() {
        // SAFETY: Registration succeeded above; failed window creation retains no HWND.
        unsafe {
            UnregisterClassW(class_name(), instance);
        }
    }
    WINDOW.store(window as usize, Ordering::Release);
    i32::from(!window.is_null())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    #[derive(Default)]
    struct State {
        calls: Vec<&'static str>,
        items: i32,
        tracks: i32,
        fail: bool,
        restored: Option<f64>,
    }
    thread_local! { static STATE:RefCell<State>=RefCell::new(State::default()); }
    unsafe extern "C" fn project(_: i32, _: *mut c_char, _: i32) -> Handle {
        ptr::dangling_mut::<u8>().cast()
    }
    unsafe extern "C" fn selected(_: Handle) -> i32 {
        STATE.with(|s| s.borrow().tracks)
    }
    unsafe extern "C" fn count(_: Handle) -> i32 {
        STATE.with(|s| s.borrow().items)
    }
    unsafe extern "C" fn cursor(_: Handle) -> f64 {
        5.25
    }
    unsafe extern "C" fn begin(_: Handle) {
        STATE.with(|s| s.borrow_mut().calls.push("begin"));
    }
    unsafe extern "C" fn insert(_: *const c_char, _: i32) -> i32 {
        STATE.with(|s| {
            let mut s = s.borrow_mut();
            s.calls.push("insert");
            if !s.fail {
                s.items += 1;
            }
        });
        1
    }
    unsafe extern "C" fn restore(_: Handle, p: f64, _: bool, _: bool) {
        STATE.with(|s| {
            let mut s = s.borrow_mut();
            s.calls.push("restore");
            s.restored = Some(p);
        });
    }
    unsafe extern "C" fn end(_: Handle, _: *const c_char, _: i32) {
        STATE.with(|s| s.borrow_mut().calls.push("end"));
    }
    fn api() -> Api {
        Api {
            main: 0,
            insert,
            begin,
            end,
            selected,
            project,
            count,
            cursor,
            set_cursor: restore,
        }
    }
    #[test]
    fn insertion_has_one_balanced_undo_block_and_restores_cursor() {
        STATE.with(|s| {
            *s.borrow_mut() = State {
                tracks: 1,
                items: 4,
                ..State::default()
            }
        });
        // SAFETY: Every function pointer is a same-signature in-memory fake; no host or HWND is used.
        assert_eq!(
            unsafe { insert_media(api(), c"C:\\test.wav") },
            protocol::ACCEPTED
        );
        STATE.with(|s| {
            let s = s.borrow();
            assert_eq!(s.calls, ["begin", "insert", "restore", "end"]);
            assert_eq!(s.items, 5);
            assert_eq!(s.restored, Some(5.25));
        });
    }
    #[test]
    fn invalid_destination_does_not_begin_or_insert() {
        for tracks in [0, 2] {
            STATE.with(|s| {
                *s.borrow_mut() = State {
                    tracks,
                    ..State::default()
                }
            });
            // SAFETY: In-memory API fakes, without any Windows/host operation.
            assert_eq!(
                unsafe { insert_media(api(), c"C:\\test.wav") },
                protocol::NO_TRACK
            );
            STATE.with(|s| assert!(s.borrow().calls.is_empty()));
        }
    }
    #[test]
    fn unsuccessful_insert_balances_undo_without_retrying() {
        STATE.with(|s| {
            *s.borrow_mut() = State {
                tracks: 1,
                fail: true,
                ..State::default()
            }
        });
        // SAFETY: In-memory API fakes, without any Windows/host operation.
        assert_eq!(
            unsafe { insert_media(api(), c"C:\\test.wav") },
            protocol::UNCONFIRMED
        );
        STATE.with(|s| assert_eq!(s.borrow().calls, ["begin", "insert", "restore", "end"]));
    }
}
