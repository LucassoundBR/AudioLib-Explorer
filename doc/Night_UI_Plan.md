# Night UI implementation checkpoints

## Delivery completed — 2026-11-16

The pending application delivery below is now complete for source8842947.
ASIO release build PASS (24m58s), license preflight/package privacy/ZIP checks
PASS, portable synthetic acceptance PASS. Installed the accepted payload into
the fixed local_release:908 package-owned files verified,4 copied, CLI1.0.0 and
isolated installed desktop startup PASS. Working libraries/preferences and the
normal data-directory selection were preserved. No release/tag was published.

Desktop SHA256:4E7E512E9E3B814FEF0BABD6B2CFE65810898A79A5EC2E24FD531483106B31C3.
CLI SHA256:45CF19DEC5C33210F59D2F4FB455369F0E07A6CF7E4877AAF0DCC15D9C1D06B1.
The installed hashes match the accepted package. Evidence remains private under
.rust-validation/night-ui/{release-resume-fixed,portable-acceptance-20261116,
local-install-20261116}.log and .rust-validation/local-install/5650eb3aedd94a97b272c414d17a87ae/report.json.

Authorized native UI verification used the same source's preview with an owned
12s/48kHz/24bit stereo file and volume0: first maximized client3840x2024 at
DPI144; Pitch dragged to12st then double-click reset0; Rate2.83 then reset1;
vertical Display Gain21dB matched Peak Display Settings, changing the settings
to7dB moved the vertical control back, then double-click reset0. Monitoring
volume remained minus infinity. The preview closed normally. Native controls
evidence: .rust-validation/night-ui/native-controls-20261116.json. The shipped
ASIO payload separately passed portable and installed startup acceptance.

The prior34 selected tests and strict desktop Clippy remain applicable: no
application source/dependency/toolchain change occurred between that accepted
checkpoint and this build. Existing playback/REAPER protocol and DLL behavior
retain the 3e35c96/ab6d89c baseline; no new hardware ASIO/listening or REAPER host
test is claimed from these layout/control checks. Initial ASIO preparation
failed before compilation because the private wrapper omitted its known local
ClangDirectory default; corrected to the previous delivery's explicit path.

Reverse is still a separate unfinished feature. Its design/export draft is
private and has not entered this installed payload.

## Reverse export development checkpoint — 2026-11-16

Added export_reversed_audio_slice for original file/range frames. PCM/float/AIFF
copies backwards in bounded256KiB blocks by complete frames, preserving channel
order, precision and rate. Compressed formats use the existing precise forward
decoder in a uniquely created owned spool, then reverse its PCM; FLAC integer
precision does not pass through f32. Source handles/version checks, create-new
destinations, cancellation and owned-output cleanup remain enforced.

Formatting and strict audio Clippy PASS;18 slice/export tests PASS (13 existing,
5 new). New checks cover channel/precision retention across block boundaries,
16/24/32-bit FLAC, PCM and compressed cancellation, existing destinations, spool
cleanup, one-frame/odd padding and double reversal. Evidence:
.rust-validation/night-ui/reverse-export-final.log. Headless preview configuration
was deliberate: export/file I/O changed; audio output, ASIO wiring, transport and
the realtime callback did not. Existing playback/backend and unchanged consumer
baselines remain applicable at this checkpoint; no dependency changed.

This is a development API checkpoint only. It is not yet connected to preview,
R, drag export or Send to REAPER, and does not replace the accepted installed
8842947 application. Next: backwards preview with original source coordinates,
direction-aware transport/selection and signed playhead interpolation, then UI
and export/send integration plus affected consumer/IPC checks.

Authorized scope: fix first maximized startup on 4K, merge REAPER controls into
the player toolbar, remove the lower zoom strip, add Pitch/Rate sliders and a
vertical visual Display Gain control. Reverse preview/export is conditional on
quota after these priorities. Keep the current Classic layout, visible controls
in FHD, keyboard shortcuts and original-audio export semantics. No unrelated
database/server work or source-audio edits.

Use one executor, targeted checks, preview builds for iteration and one final
release payload. Commit/push validated checkpoints to the existing feature
branch. Check quota at milestones; stop starting new features near 20% short
window or 85% weekly remaining. Starting observed allowance: 55%/93%. No credits
or resets are purchased. Sol 6.1 High suits DPI/Reverse diagnosis; Medium suits
the specified UI implementation. Native testing uses owned fixtures; preserve
the user's already-open AudioLib instance and its data. Use the dedicated Dummy
Audio profile if REAPER acceptance is needed.

## 4K reproduction and hypothesis

Actual monitor layout: primary 3840x2160, work area3840x2088; laptop1920x1080
offset(3840,1368). Primary application DPI144 (150%). With an explicit normal
restore size, the existing accepted executable opens with client3840x2024.
With a copy of only the user's current visual preferences (theme default,
UI scale100, maximized=true, no saved width/height), an otherwise isolated empty
library opens with WS_MAXIMIZE=true but client2160x1350. Working preferences and
library were never changed; their running application was not controlled.

Slint1.17.1 winitwindowadapter.rs checks has_explicit_size before requesting the
design preferred size after window creation, without excluding a maximized
window. A missing normal size therefore allows a preferred1440x900 logical
request to shrink the already-maximized HWND at150% DPI. The proposed fix marks
the existing design preferred restore size explicit before maximization only
when saved bounds are incomplete. Saved normal bounds and ordinary nonmaximized
startup remain unchanged. No vendor/toolchain patch or maximize/unmaximize timer.

Evidence: .rust-validation/night-ui/{baseline-geometry.jsonl,
actual-preferences-geometry.json,monitors.json}. Reproduction apps closed normally;
user application preserved.

Checkpoint 1: preview build PASS; six window_layout tests PASS (including missing
bounds, partial saved axes and existing pane/size round trips). Native first
maximized launch with the same isolated visual preferences at DPI144 now reports
client3840x2024, outer(-11,-11)-(3851,2099), WS_MAXIMIZE=true. The owned preview
was closed normally; the user's running application was preserved. Evidence:
.rust-validation/night-ui/dpi-fixed-geometry.json and check-dpi-resume.log.
Initial build attempt failed for missing LIBCLANG_PATH; resumed successfully with
the existing verified local libclang, without dependency/source changes.

This validated development checkpoint is not an installed application delivery.
Strict desktop Clippy, final UI integration and packaged acceptance remain for
the combined delivery. Existing unchanged audio, database, IPC and server
behavior retains the accepted 3e35c96/ab6d89c baseline; no related storage,
concurrency, dependency or audio-backend boundary changed in this checkpoint.

## Classic toolbar integration checkpoint

The default/Classic presentation now combines Send, Auto-Focus and Connect with
transport, numeric Pitch/Rate, their new sliders, monitoring gain and Mono. The
lower pan/zoom strip is removed; the player height absorbs the recovered Send
row so the Waveform gets that space. Display Gain occupies a narrow lane at the
Waveform's right edge and binds the existing peak-gain/peak-style callback.
Pitch/Rate retain -24..24st and0.25..4x engine limits; visual gain retains
-12..36dB. The alternate Studio presentation is preserved.

Current run: .rust-validation/night-ui/check-ui-final.log. Initial strict desktop
Clippy PASS. The initial four interaction tests failed because their generated
Slint output was still the old UI: Copy-Item preserved older draft timestamps.
The Clippy output contained the new controls while the build/test output did
not. Source timestamps were refreshed to invalidate the stale build; never
reuse that failed run as current UI evidence. Follow-up navigation/accessibility,
matching ruler bounds and obsolete zoom-bar test adaptation are now applied.
The final check includes
those changed tests, the new audition_toolbar_tests, selection gestures, S,
bilingual slider names, compact vertical visibility and wheel anchor behavior.
The synthetic 12s/48kHz/24bit stereo fixture and empty owned library are under
.rust-validation/night-ui; monitoring volume is zero. Working audio/data remain
untouched. No REAPER host acceptance is claimed from a UI-only binding check.

Checkpoint 2: formatting PASS; strict desktop Clippy PASS;34 selected tests PASS
(4 new toolbar interactions,17 selection gestures,1 S guard,6 window-layout,
1 existing themed-control interaction,1 bilingual slider check,1 pan readiness,
1 compact vertical visibility,1 wheel/ruler anchor,1 explicit visual export).
The intentional visual-export test ran separately after the normal filtered
tests; another existing ignored snapshot exporter was not requested. Classic
normal and spectral1920px renders were inspected: all toolbar controls visible,
larger full-width bottom Waveform, vertical visual gain, no lower zoom strip.
Test evidence: check-ui-final.log and ui-snapshots/classic-{normal,spectral}.png.

Quota reached4% short-window/85% weekly remaining. Stop at this validated
development checkpoint; Reverse is deferred. Preview build completion and cache
maintenance are recorded below. The final preview binary build PASS (7m24s);
it has not replaced the daily installation. No new application delivery is claimed:
release ASIO build/package, portable acceptance, final native interaction and
verified installation into local_release remain pending. The existing daily
installation and the user's running instance remain untouched. Next action:
resume with tools/build-release.ps1 after matching license preflight, package
once, validate-portable once, gracefully close the daily app in an agreed window
and install/verify the accepted payload. Private deliver.ps1 prepares the
build/package/acceptance sequence and does not install or publish anything.

End-of-checkpoint cache maintenance PASS: 40GiB debug budget met; logical
estimates59.38GiB before,33.59GiB removed,25.78GiB retained. Current target
preview and all release outputs were retained; older owned debug/host-preview
profile directories were pruned. SDKs, Cargo/Rustup home, dist, installations,
validation evidence and source audio/data were preserved. Evidence:
.rust-validation/night-ui/cache-maintenance.log. This is a logical byte estimate,
not a measurement of reclaimed physical storage.
