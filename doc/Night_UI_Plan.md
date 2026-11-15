# Night UI implementation checkpoints

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
