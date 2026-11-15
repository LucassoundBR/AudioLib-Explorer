# REAPER send integration

AudioLib sends to an already-running REAPER session through its native Windows extension.
It never launches REAPER, changes its audio driver or starts playback. Windows is
the initial supported platform. The host DLL is required; no Lua, web server or network port is needed.

## Setup and use

1. Click **Connect REAPER…** to export the bundled native DLL. With REAPER closed,
   copy it into **UserPlugins** in REAPER's resource folder, then restart REAPER
   and finish startup. This is a one-time setup; replace the DLL with the host
   closed when updating it. No script is required.
2. Keep only the intended destination instance open. Native discovery rejects
   multiple or unverifiable REAPER windows instead of guessing. The app rechecks
   process creation time, executable and HWND immediately before native send.
3. Select exactly one destination track in REAPER and place the edit cursor.
4. Select a local file in AudioLib and click **Send to REAPER · S**, or press `S`
   outside text editors. An active waveform region sends that region instead;
   wait for its original-quality export to finish preparing first.
5. Enable **Focus REAPER after send** to request foreground activation after
   REAPER accepts the import request. The default is off and the choice lasts for the
   current AudioLib session. Windows may deny activation; focus failure never
   triggers another import attempt.

The active project and selected track when REAPER processes the request are
the destination. The extension groups insertion in an Undo transaction and
restores the edit cursor. A successful acknowledgement requires a one-item count
increase; it does not return project/position telemetry, so the status does not
invent those details. Unsaved projects are supported. REAPER's media-copy
preferences still control how imported files are retained.

### Optional companion alternative

**Connect REAPER…** also generates `AudioLib_Explorer_Bridge.lua` beside the DLL
for compatibility. This is optional; native operation never loads it. Load and
run it through **Actions → Show action list → New action → Load ReaScript** if
the older explicit project/cursor acknowledgement workflow is desired.
A fresh companion for the current AudioLib data directory is selected before
submission; otherwise native discovery is used. No transport is tried after a
failed or uncertain submission to the other. The companion requires a selected
track and restores the cursor. Keep one companion session per data directory.

## Audio and file lifetime

- A whole local file is imported from its original path without re-encoding.
- A region uses the existing original-quality slice exporter, preserving sample
  rate, PCM precision and all source channels. Pitch, Rate, audition volume and
  monitoring channel choices do not alter this export.
- The region gets a durable numbered WAV in the configured slice export folder;
  it is retained after submission, including uncertain delivery. REAPER may
  reference that file, so it must not be treated as a temporary preview.
- Server-library regions use the existing verified original download/export
  path. Whole server-library files are not supported by this first version;
  select a waveform region and wait until it is ready.
- Moving/deleting an original or exported WAV later may break REAPER references,
  depending on REAPER's own media-copy preferences.

## Delivery and failure handling

Native insertion sends one bounded synchronous `WM_COPYDATA` request to the
version-checked extension endpoint, with a NUL-terminated UTF-8 path, using a two-second Windows
timeout on a worker. No input, process launch, clipboard or broadcast is used.
Timeout/zero receiver results are unconfirmed and never retried automatically.
Finish startup and close active REAPER dialogs before sending. Native readiness
checks both the main window and its last active visible popup, including the
modeless Preferences window, before publishing a durable region and again before
dispatch. Minimized main windows remain valid destinations for optional Auto-Focus.

The optional local bounded UTF-8 companion protocol uses hex-encoded paths, a session heartbeat,
single-client file lock, unique request IDs, atomic request publication/claim and
an acknowledgement with project/cursor/window identity. The client waits up to
12 seconds on a worker; no audio callback performs this work. Missing/stale
companions fall back to native discovery before making a durable region copy.
Closing AudioLib cancels
the wait and drains the owned worker.

An unconfirmed request is never retried automatically: REAPER may have imported
before its reply was lost. Check the project before manually retrying. Reconnect
after restarting REAPER. A known expired request left by a crashed client is
discarded under the send lock before a new request; it is never replayed.
This protocol is local application coordination, not a
security boundary against another process running as the same user.

## Testing boundary

File-protocol tests use temporary directories and a synthetic peer. Slint
interaction and visual tests use its offscreen testing backend, with no native
window, audio, clipboard or input injection. The Windows handle parser is tested
without invoking activation APIs.

`tools/test-reaper-companion.lua` runs the actual production companion against an
in-memory REAPER API and filesystem with Lua 5.4. It covers duplicate protection,
the cancel/claim race, cursor/undo handling, expired requests, unavailable files,
missing destination tracks, Unicode paths and unsaved projects. This checks Lua
behavior and syntax; it does not establish native API or Windows filesystem
compatibility. It can be run from the repository root with:

```text
lua tools/test-reaper-companion.lua
```

Production-script insertion and Auto-Focus require an explicitly agreed native
session or separate test machine/session. A previous native proof of concept
does not count as acceptance of a new implementation. Follow
[Shared desktop test workflow](Shared_Desktop_Testing.md); a hidden REAPER process
does not guarantee that startup notices stay in the background.

### Companion acceptance procedure

In an agreed interactive window or a separate test session:

1. Start one owned instance using the persistent REAPER test profile. Resolve
   evaluation startup normally and verify Dummy Audio in that profile. Reuse the
   same instance if initialization needs attention.
2. Open a new owned project with one selected track and edit cursor at 2.5 seconds.
   Load the **current production companion** generated by the candidate app.
   The older proof harness alone is not acceptance of this companion.
3. Send a synthetic whole 48 kHz/24-bit stereo WAV with spaces and Unicode in
   its filename, then a prepared 0.25-second region. Check destination, position,
   duration, channels/precision, source bytes,
   cursor restoration and undo. Verify errors with no track and no companion.
4. Check focus with the option off, then explicitly enable and test Auto-Focus,
   including a minimized REAPER window. Record Windows denial separately from
   insertion failure. Close owned tests normally and preserve the user's session.

Run packaged-startup acceptance before installing the candidate into
`local_release/`. CLI and offscreen checks alone do not establish native acceptance.

### Recorded native acceptance: 2026-11-14

The user authorized a shared-desktop interactive window. REAPER 7.85 used the
persistent dedicated profile, with Dummy Audio selected and verified in
Preferences. The evaluation countdown was observed and the normal enabled
Still Evaluating control used; licensing state was not changed. The normal
REAPER profile SHA256 remained unchanged.

The packaged candidate generated the actual production companion through
Connect REAPER. It was loaded as an independent ReaScript action in the owned
project. Native checks passed:

- Button and `S` send a whole synthetic WAV with spaces, accents, quotes and
  Chinese characters in its filename at the 2.5-second edit cursor. Received
  media bytes match the original 48 kHz/24-bit stereo source. Auto-Focus off
  leaves AudioLib in front. Undo removes the inserted media.
- A waveform drag prepared a 12,011-frame region (250.229 ms). `S` sent that
  region at 2.5 seconds, preserving 48 kHz, 24-bit stereo and exact source
  samples. Auto-Focus on restored the minimized REAPER window to the foreground;
  the app reported successful insertion without a focus-denied warning.
- A send without a selected track produced the expected destination-track
  error. After normal REAPER shutdown, `S` reported the missing companion and
  did not launch REAPER. Playback stayed stopped; no physical-device output
  or latency claim is made.
- Owned applications closed normally, the test-profile lease was released,
  and the original WAV and normal REAPER preferences remained unchanged.

A private inspection fixture initially grouped track creation with insertion
in one undo context and later reused a deleted track pointer. That fixture was
corrected, and the companion was loaded independently before repeating the
shortcut/region/focus checks. This was a test-fixture failure, not a production
bridge failure. Evidence is retained under ignored
`.rust-validation/reaper-native/29aa509420ce433594d9952cb4ffb041/`.

The reviewed native layout was existing `default-v2`: the Classic player has a
full-width waveform with controls above it. The earlier Studio screenshot used
the existing alternate player; this change does not move waveform/parameter
controls between those layouts. User theme preferences were not overwritten.

### Native transport application acceptance: 2026-11-15

The packaged native candidate (`35c393b`, unchanged application source after
development-tooling checkpoints) was tested without Lua or a companion, using
the persistent Dummy Audio test profile and isolated synthetic library. Full
portable startup acceptance passed. The existing button sent one whole WAV at
the edit cursor; `S` sent an active Waveform region. Both retained PCM24/48 kHz/
stereo and verified source bytes. Auto-Focus off left AudioLib foreground;
Auto-Focus on restored minimized REAPER and brought it forward. An open
Preferences dialog prevented publication, and absent REAPER was not launched.
The existing layout was retained.

**Delivery is withheld because independent Undo acceptance failed.** After the
whole-file import, Ctrl+Z and toolbar Undo did not remove the item; Edit reported
Can't undo. After the region import it again reported Can't undo, despite a
prior manual deletion and an enabled 256-MB Undo budget. The mechanism is still
under investigation. The earlier script-backed probe observation cannot
establish standalone native Undo behavior. Do not advertise equivalent Undo
semantics to the companion or overwrite the accepted `local_release/` yet.

Both owned apps were closed normally; runtime data and installed binaries were
preserved. See [native acceptance and next diagnostic](REAPER_Native_Send_Plan.md#milestone-4-application-acceptance-delivery-withheld-2026-11-15)
for exact artifacts, checks and unverified cases. A production fix requires
affected source checks, a revised release payload and independent native
acceptance before installation; workflow-only changes do not require rebuilding
the unchanged candidate.

The subsequent [Undo diagnostic/native extension proof](REAPER_Native_Send_Plan.md#milestone-5-undo-diagnosis-and-native-extension-proof-2026-11-15)
demonstrated independent insertion Undo/Redo using the public REAPER API without
Lua. It is a test-only DLL proof, not the button/S implementation. Production
integration is pending a decision on requiring a host extension installation;
the accepted daily installation is unchanged.

### Native extension integration (approved implementation)

The user approved the native DLL approach after the independent Undo/Redo proof.
The Windows build embeds `reaper_audiolib_bridge.dll` in AudioLib; no separate
download or Lua execution is needed. Connect REAPER exports the DLL into the
selected AudioLib data directory's `reaper-bridge` folder and shows its path.
Copy it into **UserPlugins** in REAPER's resource folder (Options → Show REAPER
resource path), with REAPER closed, then restart the host. This export does not
silently change a REAPER profile. The generated legacy Lua companion remains an
optional compatibility artifact; it is not required or loaded by native setup.

The original button, S shortcut, region exporter and opt-in Auto-Focus remain in
place. Without a freshly running opt-in Lua companion, the client now requires
one verified REAPER process and its versioned native extension endpoint. It checks
readiness, open dialogs and exactly one selected destination track before region
publication, then rechecks at dispatch. Missing/incompatible extensions produce
setup instructions; there is no fallback to the stock import that lacked an
independent Undo point. Uncertain delivery is never retried automatically.

The extension runs insertion on REAPER's UI thread, brackets it in a public Undo
transaction, restores the edit cursor and acknowledges only a one-item count
increase. Native status does not invent project/position telemetry. The extension
does not register a network service or perform audio callbacks. The DLL source
and desktop client share bounded local protocol definitions. The standalone fast
target compiles both actual endpoints and checks Undo transaction sequencing,
no-track refusal and balanced failure without Slint or a live host.

This section describes the implementation; final package and button/S native
acceptance results must be recorded separately before calling delivery complete.

### Accepted native DLL delivery

The packaged source `3e35c96` subsequently passed independent native acceptance
and fixed local installation. Actual button/S, Undo/Redo after saving, region
Undo preserving the previous item, cursor preservation and minimized-host
Auto-Focus passed with Dummy Audio and no Lua. Missing DLL, open Preferences and
closed REAPER were refused as expected. Exact region samples, PCM24/48-kHz/
stereo and copied-file identity were verified. The full-width Classic layout
was retained; normal preferences were not overwritten.

The accepted payload is installed in `local_release/`, and its DLL is installed
with verified SHA256 in the machine's normal REAPER resource `UserPlugins`.
The normal host was closed during installation; its INI remained unchanged.
Other installations still need the one-time DLL setup above. See [final
acceptance and limits](REAPER_Native_Send_Plan.md#milestone-7-final-package-native-acceptance-and-local-delivery)
for evidence and checks that were not exercised.
