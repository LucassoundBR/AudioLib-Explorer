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

The user requested no interface interaction on 2026-11-15. Milestone 1 can run
without windows or audio. Native insertion, receiver encoding/result semantics,
track/cursor behavior, speed and Auto-Focus remain unverified for this transport.
Do not replace the already-validated companion path until milestone 2 passes.

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
