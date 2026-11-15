# REAPER native send: staged implementation

## Goal and constraints

Send local original audio or the existing prepared original-quality slice to an
already-running REAPER on Windows, without a companion. Keep the current player
layout, Send button, S shortcut and opt-in Auto-Focus. Do not launch REAPER,
simulate input, acquire audio devices or retry an uncertain delivery.

This plan borrows research-first, explicit acceptance criteria and small reviewed
milestones from ECC/Superpowers. Existing project rules control test scope and
reuse of evidence; their global test/coverage rules and automation are not installed.

## Research and decision

REAPER 5.78 documents `WM_COPYDATA`, `dwData=0x100`, with a path and optional
`#NOTAB#START:...#LENGTH:...#` prefix. First test `#NOTAB#` plus an absolute path;
use existing prepared WAV slices rather than introduce new offset/handle semantics.

- [REAPER protocol](https://www.reaper.fm/download-old.php?ver=5x)
- [WM_COPYDATA lifetime](https://learn.microsoft.com/en-us/windows/win32/dataxchg/wm-copydata)
- [Bounded send](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-sendmessagetimeoutw)

The initial UTF-8 payload is a hypothesis requiring native Unicode acceptance.
Win32 message completion/receiver result is not a verified insertion receipt:
do not invent project, cursor or media-count data from that return value. On
timeout/ambiguous failure retain the exported WAV and never retry automatically.

## Milestones

1. **Standalone protocol probe** (`tools/reaper-native-probe.rs`). Pure tests for
   payload, path validation, explicit target identity and outcome classification;
   strict compiler/Clippy checks. Default CLI is inert; real send requires
   explicit interactive mode, PID and process creation-time identity. Commit
   this milestone without changing or rebuilding the application.
2. **Native acceptance** in an agreed idle session, using the persistent owned
   REAPER profile and Dummy Audio. Observe startup normally; bind the same owned
   process. Send 48 kHz/24-bit stereo synthetic whole and prepared-region WAVs,
   including Unicode, spaces, quotes and #. Inspect actual media count, cursor,
   source bytes, selected track, Undo, no-track behavior and modal behavior.
   No production companion may be running in this test. Record PID/start/profile
   and results; distinguish a read-only test inspector from a send companion.
3. **Application integration**, after native behavior is established. Add a narrow
   Windows adapter, fail on ambiguous instances (never choose an arbitrary
   project), revalidate target before dispatch, preserve original/slice lifetime
   and cancellation. Reuse existing controls and focus implementation. Keep the
   companion available as an explicit alternative, never as an automatic retry.
   Add meaningful dispatch/controller tests and accurate status text; commit.
4. **Delivery acceptance**. Run affected format/strict Clippy/tests, build/package
   once, portable acceptance and fixed `local_release` installation. In an
   authorized session validate S after waveform drag, focus off/on including
   minimized REAPER, no REAPER, multiple instances and no duplicate insertion.
   Commit/push the verified milestone and report remaining limits.

## Current boundary

The first milestone ran without interface interaction. The user then authorized
the native transport tests recorded below, followed by integration. After
returning to the shared computer during the release build, the user requested
CLI-only work again. Do not interact with windows or run GUI-starting CLI tests
until another agreed interactive window. The native transport is verified for
the recorded files/REAPER version; application button/S, Auto-Focus, full portable
startup acceptance and installation remain pending.

## Milestone 1 evidence (2026-11-15)

Standalone MSVC Rust 1.97.1 compilation with `-D warnings` passed; standalone
Clippy with `-D warnings` passed. Five pure tests passed: literal UTF-8 payload,
extended drive/UNC normalization, invalid/oversized input rejection, timeout and
receiver-zero classification, and inert/default/invalid CLI behavior. This does
not establish REAPER acceptance of that payload or its result-code semantics.

Build without Cargo dependencies, writing outputs only to the ignored directory:

```powershell
# Use the repository's Rust 1.97.1 MSVC toolchain bin directory on PATH.
rustfmt --edition 2024 --check tools/reaper-native-probe.rs
rustc --edition 2024 -D warnings --test tools/reaper-native-probe.rs -o .rust-validation/reaper-native-probe/probe-tests.exe
.rust-validation/reaper-native-probe/probe-tests.exe
clippy-driver --edition 2024 -D warnings tools/reaper-native-probe.rs -o .rust-validation/reaper-native-probe/reaper-native-probe.exe
.rust-validation/reaper-native-probe/reaper-native-probe.exe --help
```

For the later authorized session, `CREATION_TICKS` means the process creation
FILETIME (100-ns units since 1601), obtained from the tracked owned process with
`StartTime.ToFileTimeUtc()`, not .NET `DateTime.Ticks`. Use the existing process
identity/run record after startup readiness; the probe never starts a host or
chooses a PID itself. Its flag does not replace interactive authorization.

The application and its dependencies were not changed; existing application
validation remains applicable. This standalone experiment does not trigger an
application rebuild/package/install. At that milestone no native send or Auto-Focus was executed.

## Milestone 2: native transport acceptance (2026-11-15)

In the explicitly authorized interactive session, REAPER 7.85 ran the persistent
owned profile with Dummy Audio (verified in its audio status). The normal enabled
Still Evaluating control dismissed the evaluation notice. A new owned project
and read-only media inspector were used; no send companion was loaded.

The compiled native probe sent a synthetic 1-second 48 kHz/24-bit stereo WAV
named with spaces, accents, Chinese, quotes and #. The receiver returned nonzero;
independent project inspection established one item at the 2.5-second edit
cursor and exact original file SHA256. Undo removed the item. A prepared
250-ms WAV was then sent at the current 3.5-second cursor: one 48 kHz stereo
item, 0.25-second duration, with SHA256 identical to the prepared 24-bit slice.
Playback remained stopped and the normal REAPER preferences hash was unchanged.

Native insertion follows REAPER import preferences: this profile advances the
edit cursor to the inserted item's end, unlike the old companion's explicit
cursor restoration. Application status must not fabricate cursor/project data
that this transport does not return. Missing-track, multi-instance, modal and
Auto-Focus cases require separate application acceptance; they are not inferred
from the successful whole/region transport checks. Evidence is retained in the
ignored run `f45b6201420b4851a75151d19f74304c`.

The first Preferences-window guard experiment failed: the main HWND remained
enabled and native import succeeded while Preferences was open. The hypothesis
that enabled main HWND implied readiness was rejected. Adding a visible
last-active-popup check then refused a send with the same Preferences window
open. This extra condition is applied before region publication and rechecked
at dispatch in the production adapter. This is a recorded failed experiment,
not a passing modal test for the earlier prototype revision.

## Milestone 3: automated integration and candidate package

Commit `35c393b` integrates native discovery/import into the existing button and
S handler. Source layout is unchanged; setup help describes native use and the
optional companion. Automated checks passed as recorded in Rust_Validation.md.

The Windows x64 ASIO build passed in 28m16s. Packaging passed licenses, updater
compatibility, publication/privacy boundary and ZIP-entry checks (906 files).
Both packaged EXEs match build outputs and checksum sidecars; version-only
execution returned 1.0.0/exit 0 without initializing runtime data or GUI.
Archive SHA256: `12e52e266541447d85dca67a6132266bbdfa79d9cc38fd512850250bdb094ee9`.

The candidate is not installed over `local_release/`. Next: in an authorized
interactive window, run full `validate-portable.ps1`, then test the packaged app
against the preserved tracked REAPER instance/project with no Lua loaded.
Observe native button/S and focus off/on before raising windows; save/inspect
the owned RPP/WAVs and verify Undo. Finish fixed installation and its startup
identity checks only after acceptance. Preserve existing user sessions; recheck
the tracked PID/start/profile/project before resuming, never blindly relaunch.
