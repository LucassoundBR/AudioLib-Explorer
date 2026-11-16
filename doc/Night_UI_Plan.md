# Night UI implementation checkpoints

## Numeric units and ruler removal checkpoint — 2026-11-16

Stage3 adds an audition-local UnitNumberInput: the editable TextInput holds only
the numeric string, with a separate painted st/× suffix inside its border. Both
Classic and Studio use it; Classic keeps Enter/focus-loss submission and Studio
keeps its existing stepped NumberField wrapper. The Studio Pitch field was
widened to match Classic's76px. No shared theme input, audio DSP, persisted
setting format or REAPER transport changed.

Removed the Classic24px and Studio20px ruler rows; flexible plot height absorbs
the space. Kept current/duration counters, selection feedback, direct Waveform
seek/zoom/pan and Studio overview. Removed the obsolete Classic ruler-label
formatter. Replaced the obsolete ruler-label UI assertion with recovered-space
and numeric-input coverage, retargeted zoom checks to the surviving Waveform,
and checked drag-grip separation against the retained clock.

Formatting and desktop all-target ASIO strict Clippy passed in stage3.log.
After the final width-only adjustment and additional tests, stage3-final.log
records27 passing tests:6 toolbar/input (including numeric-only Enter submission,
typing r/s without global actions in both presenters and plot/clock spacing),
18 selection/drag/playhead tests,2 wheel/pending-render anchor tests and1 explicit
offscreen image export. Classic and Studio normal renders inspected: units
inside fields, no ruler, larger Waveform and intact counters/selection strip.
The final width adjustment is covered by this compiled interaction/render pass;
unrelated Clippy targets and audio/data-safety suites were not repeated for it.
Evidence: .rust-validation/night-ui/stage3{,-final}.log and stage3-snapshots/.

This remains a source checkpoint, not an installed application delivery. No
native windows/input/audio or working data were used. Release/package/startup/
installation acceptance remains for the combined delivery. Existing unchanged
audio/export/REAPER/database baselines retain their previous recorded scope.
Next: mirrored Reverse display with forward visual progress and correct mapping
of selection/seek/export; update the original repository last before delivery.
Suggested model for that mapping work: Sol6.1 High.

Final cache maintenance PASS in Incremental mode:18,700,325,809 logical bytes
retained (about17.42GiB), zero removed,40GiB budget met. Compiled dependencies,
release outputs, SDKs, installation and data/audio/evidence retained. Evidence:
.rust-validation/night-ui/stage3-cache.log. Quota snapshot38% short /59% weekly.

## Toolbar refinement development checkpoint — 2026-11-16

User reordered the refinement plan: toolbar first, then numeric units/ruler,
then mirrored Reverse presentation, and upstream integration last before final
delivery. Upstream refs were inspected at bbd5df2/v1.0.2; none were merged.

Classic Send is now an export icon with name/S tooltip and unchanged accessible
name, send callback, busy/modal guards and Auto-Focus checkbox. Studio's separate
Send row also uses the icon. Reverse shows its name without R on both faces;
the explanatory hover retains R. Connect was removed from both bars; Settings
now has REAPER… invoking the existing setup callback, gated by platform/busy/
background state and the existing parent menu readiness. The processing engine,
REAPER transport/DLL and selection coordinates were not changed.

Formatting and strict desktop all-target ASIO Clippy PASS. Seven selected tests
PASS: five audition-toolbar interactions (including visible Send, Auto-Focus
and output controls at1920 and1440 logical widths), one S/search/busy/modal
regression across both presenters and one explicitly invoked offscreen image
export. Classic and Studio normal renders were inspected: icon-only Send,
name-only Reverse and no Connect overflow. Evidence:
.rust-validation/night-ui/toolbar-cleanup-final.log and toolbar-cleanup-snapshots/.
The first two checks failed during UI generation on the new Studio tooltip's
styled-text/interpolation syntax; the final run corrected it to an interpolated
label property. Failed logs are retained, not reported as passing evidence.

This is a validated source checkpoint, NOT an installed delivery. Native hover
and Settings-menu acceptance remain for the final GUI window, alongside the
release/package/portable/install sequence. local_release and the accepted Reverse
payload remain unchanged. Prior audio/export/native REAPER and data-safety
baselines remain applicable to their unchanged implementations; unrelated
subsystem tests were excluded. No native GUI, device output or user-data writes
were performed. Next: put st/× inside the numeric fields and recover the ruler
height for the Waveform; preserve existing numeric parsing/reset behavior.

Final cache maintenance PASS:40GiB budget met. Logical estimates46,299,661,920
bytes before,43,372,743,013 removed,2,926,918,907 retained. Both owned preview
profile directories were pruned as coarse budget candidates; subsequent preview
checks may require cold compilation. Release outputs, installed application,
SDK/toolchain, validation evidence and working data/audio were retained. Evidence:
.rust-validation/night-ui/toolbar-cache-maintenance.log. These are logical byte
estimates, not a physical free-space measurement.

## Reverse installed and accepted — 2026-11-16

Reused the exact cb56405 ASIO package above, without compilation. Runtime
playback/IPC acceptance and portable acceptance PASS with isolated synthetic
fixtures. Evidence: .rust-validation/night-ui/reverse-runtime-ipc.log and
reverse-portable.log. Existing scoped source/DSP/export/Send tests remain
applicable because no application source, feature or toolchain changed.

Authorized native acceptance PASS: R enabled Reverse and moved the whole-file
cursor to12s; muted playback moved backwards (observed3.613s). Clicking Reverse
disabled it. A mouse-selected region retained its interval after R, prepared
the reversed WAV, and Space responded. Confirmed engine status recorded
reverse=true, volume=0, no error, zero underruns, range159482..329143 at48kHz,
Ended at3.3225416666666665s, exactly its first source frame. Reverse=true and
volume0 persisted after normal close. Evidence: reverse-native-status.json,
reverse-native-visible.log and the owned baseline/settings.json. No listening,
latency measurement or fresh native REAPER/Auto-Focus test is claimed; native
host/DLL behavior is unchanged and retains its prior accepted baseline.

The first owned fixture launched outside the tool's visible desktop. After
normal close was unavailable, only that identity-checked disposable fixture
process was terminated, then relaunched on the visible desktop. The working
AudioLib closed normally before testing; the visible fixture closed normally.
An initial non-elevated IPC status probe was denied; the matching-context probe
succeeded and supplied the confirmed state above.

Installation PASS:908 package-owned files verified,5 copied, CLI1.0.0 and isolated
installed startup passed at fixed local_release. Installed hashes match the
candidate hashes below. Receipt:
.rust-validation/local-install/538698fd717b4f348a48a0d731132ae5/report.json.
Working libraries, Metadata, preferences and normal data-directory selection
were preserved. No Rust compilation/tests were repeated; previous budgeted
cache maintenance remains valid and was not rescanned for this acceptance-only
step. No release/tag/merge was published. Next: normal user audition of the
installed Reverse; changes require new scoped checks, not a routine rebuild.

## Reverse candidate prepared; native acceptance deferred — 2026-11-16

Source cb56405 (audio checkpoint bb84a12) passed the scoped checks recorded below.
The Windows x64 ASIO release build PASS (27m29s), followed by matching license,
privacy, updater-file-policy and ZIP checks PASS (906 ZIP entries). Candidate:
dist/latest/portable and dist/latest/AudioLibExplorer-latest-win-x64-portable.zip.
The packaged CLI reports1.0.0. No release, tag or merge was published.

Candidate desktop SHA256:C616DF97641481A5D14CC7A26BC1B5BFAF6886C47AF729F616A9F244E8F14DC4.
Candidate CLI SHA256:A9D30BE4E8C58519C185DEB4EF769D723A20991F1E1B9F39575D607689503C0C.
Archive SHA256:6B4BD83AF395C5E90423D3FDA930D536F5806FDCE3C58E62E6EB89CAC9F93018.
Evidence: .rust-validation/night-ui/reverse-release.log. Classic normal/spectral
and alternate Studio normal renders were inspected; the added Reverse control
fits. This is rendered/testing-backend evidence, not native input acceptance.

The user had authorized interactive validation, but Computer Use then reported
that physical Escape stopped control. No further Computer Use or GUI launch
was attempted. Reverse native interaction, real runtime IPC/playback, portable
startup acceptance and installation remain NOT RUN. In particular, no new
REAPER-host, Auto-Focus, physical audio or hardware-latency result is claimed.
The accepted8842947 application remains installed at fixed local_release, with
its desktop hash freshly confirmed as4E7E512E9E3B814FEF0BABD6B2CFE65810898A79A5EC2E24FD531483106B31C3.
Working applications, normal data selection, libraries and source audio were
not changed by this candidate build. Reverse is not an installed delivery yet.

Next, in a newly authorized interactive window, reuse this exact candidate:
gracefully close the normal AudioLib if required; run validate-playback-ipc.ps1
with BinaryDirectory dist/latest/portable and CleanCheckout; run
validate-portable.ps1 against that directory once; verify native R/button,
backwards source/range progress, selection export and settings retention with
the owned muted fixture; then install-local-release.ps1 and verify identity/
isolated installed startup. Reuse the dedicated Dummy Audio profile for any
new REAPER host/focus test, not the normal profile. Do not rebuild or repeat
passing source checks unless relevant source/features/toolchain changed.
Sol6.1 Medium is sufficient for this acceptance/install sequence; use High if
an actual new transport/DSP defect requires diagnosis.

End-of-checkpoint cache maintenance PASS:40GiB development-cache budget met;
40,198,770,169 logical bytes retained (about37.44GiB), zero removed. Release,
SDKs, Cargo/Rustup home, installations, data/audio and evidence were preserved.
Evidence: .rust-validation/night-ui/reverse-cache-maintenance.log. Quota snapshot:
17% short-window /71% weekly remaining; no new features started at this boundary.

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

## Reverse preview and integration verification — 2026-11-16

Backwards decoding uses bounded worker-side blocks before resampling/SoundTouch,
keeping original source coordinates, channel order and range boundaries. The
transport uses signed progress and direction-aware start/end, seek, Stop and
repeat. Existing serialized audition settings default Reverse to false.
Buffered server streams open with Reverse disabled; local originals are required.
No decoder I/O or allocation moved into the realtime callback.

ASIO audio strict Clippy and35 selected audio tests passed before integration:
4 Reverse decoder/transport/compatibility tests, the existing audition test,
10 range tests,6 transport tests and14 seek tests. Evidence:
.rust-validation/night-ui/reverse-playback.log. These are decoder/worker proofs
without CPAL streams, listening or hardware latency measurements.

The application adds a Reverse button and R, signed playhead interpolation,
whole-file/selection reverse export and the existing drag/Send path. Whole-file
reverse derives its bounds from the locked original header. Changing direction
retires only the prepared export, retaining selection identity and preview.
Send waits for the acknowledged direction and newly prepared WAV. REAPER DLL,
native transport, Undo/Redo and focus protocol are unchanged from the accepted
3e35c96/ab6d89c baseline; that baseline is not a fresh host test.

Integration checkpoint PASS: formatting; strict all-target ASIO Clippy for
audio, IPC, CLI and desktop;18 export tests;5 Reverse tests including the added
MP3/Ogg/FLAC block-boundary comparison;1 IPC contract test;27 selected desktop
tests (5 toolbar,18 selection UI,1 S guard,1 prediction,1 isolated Send and1
explicit visual export). The isolated Send peer verified exact original and
reversed whole/range bytes, stale-direction refusal, retained interval/preview
identity, durable export retention and close cancellation. No native host or
audio device was used by these tests. Classic normal/spectral1920px renders
were inspected: controls fit together, bottom Waveform and vertical gain remain.

Initial consumer Clippy found one redundant String conversion in progress
reporting; corrected and resumed successfully. Evidence:
.rust-validation/night-ui/reverse-integration-fixed.log and reverse-ui-snapshots/.
The prior31 audition/range/transport/seek regressions remain applicable because
their decoder/transport implementation did not change after those passing
checks; only an additional test and whole-file export API were added. Database,
server, library writes, shutdown ordering, device backend and dependencies are
unchanged; unrelated subsystem suites were excluded. Native UI/runtime,
accepted release payload and installation are still pending; local_release
continues to contain the accepted8842947 UI build.

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
