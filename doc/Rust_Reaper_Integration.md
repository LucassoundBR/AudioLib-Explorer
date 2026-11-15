# REAPER send integration

AudioLib sends to an already-running REAPER session through native Windows import.
It never launches REAPER, changes its audio driver or starts playback. Windows is
the initial supported platform. No extension, web server or network port is needed.

## Setup and use

1. Open one REAPER instance and finish startup. No script is required.
2. Keep only the intended destination instance open. Native discovery rejects
   multiple or unverifiable REAPER windows instead of guessing. The app rechecks
   process creation time, executable and HWND immediately before native send.
3. Select a destination track in REAPER and place the edit cursor.
4. Select a local file in AudioLib and click **Send to REAPER · S**, or press `S`
   outside text editors. An active waveform region sends that region instead;
   wait for its original-quality export to finish preparing first.
5. Enable **Focus REAPER after send** to request foreground activation after
   REAPER accepts the import request. The default is off and the choice lasts for the
   current AudioLib session. Windows may deny activation; focus failure never
   triggers another import attempt.

The active project and selected track when REAPER processes native import are
the destination. Import follows REAPER preferences, including cursor advancement
and behavior when no track is selected. A completed native message does not
provide project/cursor/item-count telemetry; the status reports acceptance of
the request without inventing an insertion receipt. Unsaved projects are supported.

### Optional companion alternative

**Connect REAPER…** still displays the optional companion path. Load and run it
through **Actions → Show action list → New action → Load ReaScript** if its
explicit project/cursor acknowledgement and cursor preservation are preferred.
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

Native import sends one bounded synchronous `WM_COPYDATA` request with `dwData`
`0x100`, a NUL-terminated UTF-8 path and `#NOTAB#`, using a two-second Windows
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

### Native acceptance procedure

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
