# Shared desktop test workflow

The user may be working in other apps while Codex develops AudioLib Explorer.
Hiding a GUI process does not guarantee that its startup dialogs, plugin scanners
or device initialization stay in the background.

## Default: preserve the user's desktop

- Run CLI, file, protocol and DSP checks without taking focus, injecting mouse or
  keyboard input, changing the clipboard or playing test audio.
- Do not start REAPER on the shared desktop while the user is working. Defer its
  native checks; do not describe deferred checks as passed.
- GUI automation on the same Windows input session can interfere with the user.
  For unattended GUI tests while the user works, use a separately provisioned
  test session, machine or VM with its own input surface. None is provisioned by
  this change. Real device/ASIO acceptance still needs the relevant hardware.

## Persistent REAPER test profile

Use `.rust-validation/reaper-profile/reaper.ini` instead of creating a fresh
resource profile for every test. Driver preferences and plugin caches persist;
projects, WAVs, queue files and reports stay in separate run directories.
The harness locks the profile and refuses a second tracked active instance.

Prepare files without opening REAPER:

```powershell
pwsh -NoProfile -File tools/test-reaper-send.ps1 -PrepareProfile
```

On the first preparation, `-SeedRun <owned-successful-run-id>` can copy the
allowlisted preferences and plugin caches from a closed, verified test run.
It does not copy registration files or reset an existing profile. File preparation
does not establish that native startup or all dialogs are ready.

## Agreed interactive window or isolated desktop

For insertion/protocol and offline rendering tests, prefer REAPER's Dummy Audio
backend in the dedicated profile. It avoids acquiring the user's hardware audio
interface or exclusive ASIO driver. Configure and verify the actual backend in
an authorized setup session; do not guess numeric `reaper.ini` values or change
the user's normal profile. On 2026-11-14, Dummy Audio was selected and verified
in Preferences in the persistent `.rust-validation/reaper-profile` during an
authorized session. Recheck the backend if that profile is replaced. Dummy
Audio does not prove real-device latency/playback and does not
remove the evaluation notice, update dialogs or plugin-licensing prompts.

Production sends use native Windows import in an already-running REAPER, with
the loaded companion retained as an optional alternative. AudioLib never launches
REAPER or changes its audio driver. The older `test-reaper-send.ps1 -Run` exercises
the companion proof, not the native import adapter. For native acceptance use
the [native plan/probe](REAPER_Native_Send_Plan.md) and then the actual packaged
button/S/focus workflow in the dedicated profile. A packet unit test or companion
pass does not establish native application acceptance.

After the user has agreed to interactive testing, run:

```powershell
pwsh -NoProfile -File tools/test-reaper-send.ps1 -Run -Interactive
```

Auto-Focus is a separate test:

```powershell
pwsh -NoProfile -File tools/test-reaper-send.ps1 -Run -Interactive -TestFocus
```

These flags express test mode; they do not replace user authorization to use the
shared input session. `-SkipFocus` remains accepted for older callers, but focus
testing now requires explicit `-TestFocus`.

During authorized setup, handle the audio-device prompt in the owned profile
without changing the user's normal REAPER settings. For the evaluation notice,
observe the countdown, wait until Still Evaluating is enabled, then use that
normal control. The persistent profile does not guarantee that licensing notices
will disappear. Never alter licensing state or bypass a countdown.

Use supported computer-use tools to handle known dialogs in the authorized
desktop. The harness itself waits for the bridge acknowledgement; it does not
click dialogs. On timeout, keep the process identity and resume the same run
with `-ResumeRun <id> -Interactive` once startup is resolved. Do not repeatedly
launch fresh instances. Unknown dialogs require diagnosis; no blind keystrokes.

## Validation boundary

Policy/preflight/profile checks can run without GUI. Changes to this workflow do
not rebuild application binaries. Native behavior and first-run dialog handling
require an authorized interactive window or isolated desktop. The 2026-11-14
authorized session verified Dummy Audio, normal evaluation-notice handling and
the current packaged app's production companion insertion/Auto-Focus, including
minimized-window restoration. See [recorded native acceptance](Rust_Reaper_Integration.md#recorded-native-acceptance-2026-11-14).

In this environment a sandboxed GUI launch initially ran on a desktop unavailable
to the UI-control tool. A process existing or having a window handle is not a
readiness handshake. Its exact owned PID, start time, executable and profile were
verified before ending that uninitialized launch; the same run files and profile
were resumed on the interactive desktop. Do not launch repeated instances to
diagnose desktop visibility. Reuse the active instance after resolving known
startup notices. The production app itself never launches REAPER.
