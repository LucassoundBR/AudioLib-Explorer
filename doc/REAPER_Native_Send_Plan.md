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
cursor and exact original file SHA256. Undo removed the item in the inspector
fixture, but the later independent application check below invalidates treating
that result as native Undo acceptance. A prepared
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

## Milestone 4: application acceptance, delivery withheld (2026-11-15)

The authorized interactive session reused the persistent Dummy Audio profile
and an owned project, with no Lua or companion loaded. The same candidate from
`35c393b` was used; later development-tooling commits did not change shipped
application source or the release profile. No rebuild was needed.

Full portable acceptance passed once: EXE identity, isolated first startup and
synthetic scan, CLI statistics, Chinese UI startup/exit, expected invalid-settings
diagnostic, licenses and unchanged source. The candidate desktop SHA256 remains
`f77d39f993774dd073be42b82bd3fc1ec82fbd9a7482728eade32beb56280bb6`.

Application checks established:

- Button, focus off: exactly one 1-second item at cursor 2.5 seconds; AudioLib
  remained foreground. REAPER copied the media according to its preferences;
  the imported file SHA256 matches the original 48 kHz/24-bit stereo source.
- `S` with an active Waveform region: exactly one 12,015-frame/250.3125-ms item
  at the then-current cursor of 3 seconds. Auto-Focus restored minimized REAPER
  and brought it forward, observed before any manual activation. Export and
  imported copy have identical SHA256, retain PCM24/48 kHz/stereo, and their PCM
  matches a contiguous source segment exactly. This periodic sine fixture has
  multiple identical segments, so byte matching alone does not prove selection
  start accuracy. The UI displayed approximately 0.249–0.499 seconds.
- Preferences open: `S` refused before publishing another durable slice; export
  count remained four and the saved project hash was unchanged.
- REAPER closed: `S` requested opening REAPER and did not launch another host.
- **Undo failed.** After the whole import, Ctrl+Z and the Undo toolbar did not
  remove it; Edit reported Can't undo. The item was removed explicitly only
  from this owned synthetic project to isolate the region test. After the
  region import, Edit again reported Can't undo, although a manual deletion had
  previously created an undo point. Preferences showed a 256-MB Undo budget,
  rather than disabled Undo. This suggests history invalidation during import;
  its mechanism and version dependence remain unproven.

The earlier inspector-backed Undo result is insufficient: it did not establish
the behavior of a standalone native import without a script context. Do not
claim native Undo or complete delivery on that evidence. `local_release/`
retains the previously accepted companion build. No installation, tag or release
was performed. The layout and normal runtime data were not changed. Owned apps
closed normally and the profile lease was released.

Next: reproduce import/Undo with a pre-existing ordinary edit in an owned
project, comparing a simple ASCII WAV and the Unicode/# fixture, before and
after saving. Compare documented native packet options in an isolated probe
against the [official transport example](https://www.reaper.fm/download-old.php?ver=5x);
do not assume an option fixes Undo or add compensating synthetic edits. If the
native protocol cannot preserve history, evaluate a native REAPER extension
before changing the agreed integration architecture. Reuse the accepted package
startup result if artifacts stay unchanged; a production fix requires affected
checks, a new release payload and fresh native/installation acceptance.

Private evidence: `button-whole.rpp`, `button-undo.rpp` (still contains one item;
the filename denotes an attempted Undo), and `s-region.rpp` in owned run
`f45b6201420b4851a75151d19f74304c`; `wave-acceptance.json`,
`popup-before.json`, `application-acceptance.json` and `portable-final.log` in
`.rust-validation/reaper-native-probe/`. Real multiple-instance ambiguity and
foreground-denial cases were not exercised in this session.

## Milestone 5: Undo diagnosis and native extension proof (2026-11-15)

An authorized diagnostic session isolated the missing Undo entry from filename
and save behavior. In the same persistent Dummy Audio profile, adding a track
normally produced an enabled Undo "Add new track". A one-second ASCII WAV sent
through the existing `0x100` transport inserted successfully but left that
previous Undo entry unchanged, before saving. An isolated probe using documented
`START:0`/`LENGTH:1` options behaved the same. Invoking Undo from the visible menu
then removed the previously created track and its imported media. Therefore the
observed defect is a missing independent import Undo entry; the earlier inference
that native import necessarily clears all history is not established. Keyboard
attempts alone were not used as proof of the resulting project state.

`tools/reaper-undo-extension.rs` is a **standalone experimental extension**, not
part of the application build/package or daily REAPER installation. It uses the
[official extension ABI](https://github.com/justinfrankel/reaper-sdk/blob/main/sdk/reaper_plugin.h)
and host-resolved `InsertMedia`, `Undo_BeginBlock2`/`Undo_EndBlock2`, project/item
queries and cursor APIs. A private hidden receiver executes one synchronous
transaction on REAPER's main thread. It checks a bounded NUL-terminated absolute
WAV path, exactly one selected track, main-window readiness and reentrancy. It
does not use Lua, simulate edits, inject input, activate windows or run a network
listener. This proof requires installing a DLL in REAPER and restarting the host;
that is an integration/setup tradeoff, not equivalent to zero host setup.

Strict standalone Clippy and compilation passed without Slint. The DLL was
loaded only from the dedicated profile's `UserPlugins/`. Whole-file and region
requests used explicit owned PID/start identity through a private adapted probe.
No production companion or inspection script was loaded. Native menu actions and
independently saved RPP snapshots established:

| State | Tracks | Items | Edit cursor (s) |
| --- | ---: | ---: | ---: |
| Whole insert, saved | 2 | 2 | 5.25031250005122 |
| Undo whole, saved | 2 | 1 | 5.25031250005122 |
| Redo whole, saved | 2 | 2 | 5.25031250005122 |
| Region insert, saved | 2 | 3 | 5.25031250005122 |
| Undo region, saved | 2 | 2 | 5.25031250005122 |

The pre-existing item and ordinary track-creation history survived the extension
Undo; the menu exposed the insertion-specific description and corresponding
Redo. The whole request used the Unicode/quotes/# source; the region used the
previously accepted 12,015-frame PCM24/48-kHz/stereo export. Playback remained
stopped with Dummy Audio. Owned REAPER closed normally. Evidence is
`undo-extension-results.json` and `extension-*.rpp` in the existing ignored run.

This establishes the native extension approach, **not a delivered app fix**.
The existing button/S still uses the earlier transport. Daily installation and
UI are unchanged. A preference question is pending because the proof introduces
a host DLL dependency: integrate the extension with explicit setup, or continue
researching a solution with no host installation. Do not silently install a DLL
in the user's normal profile. Before shipping, the receiver/client protocol,
lifecycle, failure cases, installation/update path and package acceptance need
production integration and verification. No whole desktop rebuild is justified
until that implementation is ready.

Reproduce compilation in the configured project toolchain:

```powershell
rustfmt --edition 2024 --check tools/reaper-undo-extension.rs
clippy-driver --edition 2024 --crate-type cdylib -D warnings tools/reaper-undo-extension.rs -o .rust-validation/reaper-native-probe/reaper_audiolib_undo_proof.dll
```

## Milestone 6: approved extension integration checkpoint

The user selected native DLL integration, including its one-time host setup.
Production source is now `crates/audiolib-windows/src/reaper_extension_host.rs`;
the standalone tool imports that actual source. A shared protocol module is used
by the receiver and desktop adapter. The Windows build compiles a small native
DLL into OUT_DIR and embeds it in AudioLib. Connect REAPER exports it atomically
and shows copy-to-UserPlugins/restart instructions in the existing setup dialog.
The waveform, controls, Send button, S shortcut and focus option retain their
layout. The existing Lua companion remains an explicitly running compatibility
alternative, not a requirement for the new setup.

Native discovery now requires the versioned endpoint in the verified REAPER PID.
It refuses absent/incompatible extensions and requires exactly one selected
track before durable region publication, with readiness rechecked at dispatch.
The packet is a bounded literal UTF-8 path, with no stock `0x100` fallback.
One synchronous host transaction inserts, restores the cursor and records Undo;
acknowledgement requires the expected one-item increase. Failure/timeout is never
automatically retried. Missing DLL setup cannot silently use the defective old
transport. The main REAPER HWND remains the optional foreground target.

Checkpoint validation passed on Windows x64/Rust 1.97.1:

- Actual-source fast target: strict Clippy and 11 tests, including balanced Undo
  sequencing, cursor restoration, invalid track selection and failed insertion
  without retry. The first run found test-module ordering/duplicate-import lint
  issues; those were corrected. Its deliberate import of one wire source into
  independently compiled endpoints is allowed only in the test harness.
- Root formatting, standalone extension strict Clippy/rustfmt, desktop/Windows
  strict all-target Clippy with ASIO: pass (Clippy 6m14s).
- Six Windows tests: pass, including atomic export preserving unrelated files
  and retaining a locked previous DLL on failure. Six desktop REAPER tests:
  pass; two explicit helpers remain ignored, with the child test run by its
  parent. The desktop binary retains search/edit/modal/busy guards and existing
  Controller export/close coverage.

The first SDK preparation attempt used the default LLVM path and failed before
checks; the configured verified project libclang path resolved it. Unfiltered
offline metadata inspection encountered an uncached nonselected dependency;
actual Windows checks ran offline successfully. The only dependency change is
an edge to already locked tempfile 3 for atomic DLL export, not a version update.
No schema, database, server or device-backend behavior changed. Their recorded
baselines remain applicable; unrelated regressions/hardware tests were excluded.

Evidence: `check-dll-integration.log` and fast-check run
`45b0fb6cae024a0588c233356bc96ee0`. This is a validated source checkpoint, not an
application delivery. A new release build/package, full portable acceptance,
real button/S/Undo/Redo/focus acceptance with the production DLL, fixed local
installation and final cache maintenance remain pending. The old package must
not replace the daily installation. No release/tag/merge is requested.
