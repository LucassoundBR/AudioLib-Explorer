# Upstream v1.0.2 integration — 2026-11-16

Verification continued on 2026-11-17. This record separates source acceptance
from native host/package acceptance and retains failed experiments.

## Scope and identities

Integration branch: `codex/integrate-v1.0.2`. Custom baseline: `691b891`.
Upstream input: tag `v1.0.2`, commit
`702366c391d714ea197787e87cd7e25eaf7808b3`; common ancestor `fa7c0a7`.
This integrates the tagged release, not later upstream main changes.

Keep the user's compact transport toolbar, icon-only Send with tooltip,
Settings → REAPER connection entry, units inside Pitch/Rate inputs, vertical
Display Gain and ruler-free Waveform. Reverse keeps original source coordinates
for selection, seek and export while mirroring the image and moving the visible
cursor left to right. Native REAPER DLL delivery remains in place without Lua.

Upstream adds playback buffering and IPC publication fixes, favorites persistence,
Metadata decoding/import corrections, remote credential/request safety, AI relay,
theme/scaling/keyboard corrections and build-tool routing improvements. See
[1.0.1](Rust_Release_1.0.1.md) and [1.0.2](Rust_Release_1.0.2.md) records for upstream
details; their historical test receipts are not fresh tests of this merge.

## Merge decisions

- `audition-panel.slint`: retain custom Reverse; include upstream checkable Mono.
  A duplicate Classic Reverse `checkable` binding initially failed Slint build;
  remove that duplicate before verification.
- `AGENTS.md`: retain custom fast-preview/checkpoint guidance and combine it with
  upstream build routing/delivery guidance; remove repeated installation intro.
- No product-choice conflict identified. Optional stop-on-focus-loss remains
  disabled by default. Native repeat-key forwarding is limited to Space/Enter;
  S/R continue through the existing shortcut guards.
- Use the existing owned `target/` via explicit `CARGO_TARGET_DIR` to reuse
  caches; do not mutate global Cargo configuration or move working audio/data.

## Fresh verification

Windows x64, Rust 1.97.1 MSVC, Slint 1.17.1, verified ASIO SDK 2.3.4,
`preview` profile. The integration changes multiple crates and dependencies, so
the initial Clippy check deliberately selects the whole workspace with desktop
and CLI ASIO features. GUI checks use the offscreen Slint testing backend only.
The same combined selection failed test linking in `audiolib-ai`: Cargo unified
`audiolib-windows/asio`, but that test executable does not link CPAL/asio-sys.
Retain this failed result; it is not a product runtime failure or a passed suite.
Execute desktop/CLI/audio tests with ASIO together, and the remaining subsystem
tests separately without that unrelated feature. No production source change is
needed for this test-selection issue; reuse the completed strict Clippy result.

- PASS: formatting and workspace strict Clippy, all targets.
- PASS: formatting and desktop strict Clippy after the correction batch.
- Initial desktop run: **691 passed, 12 failed, 235 ignored**. The completed
  ASIO harness from the first workspace compilation was reused directly; its
  fingerprint records `asio/default`, and `--list` reports 938 tests. Three
  HTTP/AI cases passed individually after the highly parallel run; retain both
  results rather than hiding the original timing failures. Nine failures were
  reproduced individually and are addressed below. No failed broad suite is
  presented as passing.
- PASS: final desktop correction filters (14 tests across the selected filters,
  including the explicit visual export), original partial-repaint budget,
  Classic keyboard release/focus cancellation, scaled/compact footer geometry,
  mirrored selection/seek/zoom and forward Reverse cursor. Final Classic PNGs
  were inspected; no native user window was opened.
- PASS: audio package 202 tests (4 ignored), CLI 15 unit tests and 5 remote tests.
  Audio ignores include isolated helper/fixture-generation cases and unavailable
  authentic BOOM WAV fixtures. The legacy `test_audiolib/P-51 Mustang` directory
  is absent, so `validate-runtime.ps1` was not attempted.
- Supporting subsystem run passed AI, core, database, IPC, remote and the server
  targets preceding `search_metadata_complete`; it then stopped on the stale
  identity-error expectation described below. Preserve that failed run. Only
  that integration target and subsequent unrun server targets are resumed.
- PASS: updater and Windows lib harnesses compiled during that supporting run
  were reused directly (no production feature/source changes). They have no
  separate integration-test directories. Windows harness has no ASIO feature.
- PASS: strict server Clippy, all targets, no default features, after its test fix.
- PASS: corrected `search_metadata_complete` integration target (all 5 tests),
  then the previously unrun `security`, `shutdown_cleanup`, `smed_artwork`,
  `startup_upgrade`, `stats_cache`, `updates` and `waveform_jobs` targets. Their
  declared helper/authentic-fixture ignores remain ignored, not passed.
  Source verification is complete with the environment/native limitations below.
- PASS: `test-build-environment.ps1`, `test-validator-build-routing.ps1`,
  `test-development-pipeline.ps1`, `test-release-pipeline.ps1`,
  `test-build-artifacts.ps1`, `test-package-boundary.ps1`,
  `test-server-package.ps1`.
  Build-artifact tests explicitly skipped their optional real pipeline cleanup
  integration while another compiler was active; synthetic cases passed.
- PASS: staged and unstaged `git diff --check` before final documentation edits.
- PASS: `validate-library-management.ps1` using the unchanged CLI 1.0.2 built
  during the first integration pass (preview; SHA256
  `F1FDF392453B884FE37A9C9F78B09F2A78264F5EEEE9C5910A08C902637CF376`).
  This is CLI/source acceptance, not release-package acceptance. Fixtures were
  isolated; source WAV bytes were unchanged, with no real library or audio output
  opened. Keep the new receipt private and restore the historical public report.
  Later target selection rebuilt the retained CLI with a different dependency
  feature closure; its version remains 1.0.2 and SHA256 is
  `C10F5189B34BC3857C4E5C715EA7AA1A54CD36076E577C6BCA91CC1F20BD5083`.
  Repeat library-management acceptance against that actual retained EXE: PASS,
  with the same source/audio isolation checks. This replaces its earlier artifact
  receipt for the final retained CLI; it is not a daily installation.
- INCOMPLETE: `validate-review-fixes.ps1 -CleanCheckout` stopped at file-symlink
  creation (line153): Windows requires a privilege unavailable to this process.
  Earlier checks ran, but the suite and its later cases are not approved. Do not
  change Windows policy or remove these cases to claim success.

Private logs are retained in `.rust-validation/night-ui/`; never publish fixtures,
runtime settings, credentials or raw private validation output.

The accepted evidence is compositional: reuse unchanged passing targets from
the initial broad run and each corrected targeted run. Do not describe the first
failed broad command as a fresh all-green workspace suite. The final server run
also supplies current no-default-feature Clippy evidence; the desktop keeps its
ASIO Clippy/interaction evidence. No new 1M-record benchmark, hardware latency,
native focus or package acceptance result is claimed.

## Corrections found during integration verification

- Preserve an empty selection identity when adding the Reverse presentation
  suffix. A suffix by itself had falsely enabled source selection, intercepting
  legacy seek gestures before any selectable source was published. Existing
  Studio cancellation/progressive-image seek regressions reproduce this.
- Apply the custom Classic dock height to Default v2 as well as Classic, retaining
  the enlarged Waveform when switching workbench themes.
- A fixed-size clock wrapper inside the layout did **not** fix redraw cost:
  the original repaint-budget regression still failed with identical damage.
  An isolated offscreen probe then compared direct Classic versus the facade,
  smooth prediction on/off, and clock updates on/off. The full-plot damage
  followed clock updates in both presenters, independently of prediction.
  Paint the clock outside the shared layout at its existing footer coordinates,
  reserving the same column. In the probe, clock-update frames dropped from about
  264k pixels to at most 10.3k; this is a synthetic diagnostic, not a hardware
  performance measurement. Keep the full application's original budget test.
- Update test fixtures to hit row content after the favorites cell, traverse the
  added Classic Pitch slider and enabled Reverse, expect the customized spectral background, and
  cover all eight upstream recommendations in Main-fallback batch caching.
  Their behavioral assertions remain in place; do not remove safety tests.

Separating all client roots after the initial ASIO link failure changed the
unified dependency features and began an avoidable second desktop compilation.
That owned runner/compiler subtree was identified and stopped. The first
completed test harness was retained and executed. The corrected source now uses
one desktop-only compilation and targeted regression filters; untouched passing
desktop cases are not rerun merely to refresh their date. Direct Cargo commands
use the shared lease and the scoped build environment helpers.

The first correction batch passed double-click preservation, multichannel lanes,
all eight Main-fallback searches, all four Studio seek/mute/navigation cases,
seven other Studio theme cases, five auxiliary player cases, nineteen selection
cases, S shortcut guards, seven window-layout cases and the explicit mirrored
visual test. The remaining Classic keyboard fixture stopped on Reverse instead
of Repeat; update its traversal to include both new preceding controls. Final
verification follows the hoisted-clock and traversal corrections; earlier
failed redraw and keyboard results remain in the private logs.

The supporting run also reproduced an upstream stale assertion in
`search_metadata_complete`: a request explicitly sending `wrong-library`
expected `LIBRARY_ID_REQUIRED`. The documented contract in `doc/api/README.md`
distinguishes absent identity from a mismatched one (`LIBRARY_CHANGED`), and the
server already followed that contract. Expand the regression to assert both
missing and incorrect headers with HTTP409; do not weaken server guards. No
server production code was changed. Reuse all previously passing targets and
run the corrected integration file plus only the targets after that failure.

## Build-cache integration and maintenance

The upstream Intermediate cleaner initially omitted the custom `preview`
profile from its candidate/evidence/resume lists. Its first real pass removed
2.72 logical GiB from release intermediates/tmp but left 80.98 GiB. This was
incomplete maintenance for our development profile, despite that command's
successful exit status.

Include host/triple `preview` alongside debug/release in those narrow lists,
retaining the same ownership, reparse-point, process and allowed-path guards.
The old preview regression also needed an explicit fixture target after upstream
Windows target routing changed. Initialize a separate fixture Git repository so
external-cache registration and leases cannot pollute this checkout's registry.

Fresh `test-preview-cache.ps1` and `test-build-artifacts.ps1` both PASS after this
fix. Coverage includes read-only plans, preview intermediates/tmp removal,
retained EXE/PDB and unknown files, registered untagged Cargo evidence, repeat
Intermediate runs, refusal to escalate that proof to All, and pipeline cleanup
recovery. The optional actual synthetic pipeline-cleanup integration now ran and
passed (it was deferred during the earlier active compiler).

Recovery maintenance with the corrected rules removed another **79.10 logical
GiB**, retaining **1.88 GiB** of final outputs/other preserved cache contents.
Combined logical removal was about **81.82GiB**. These are logical file-size
estimates, not measured physical free-space recovery. SDKs, Cargo/Rustup home,
dist, local_release, libraries/audio and validation evidence were preserved.
No further Rust compilation was needed for this PowerShell-only correction.

## Source checkpoint boundary

At the source integration checkpoint, no daily installation, release tag,
GitHub Release or upstream push was implied. The previous accepted 1.0.0
installation remained in place until the final acceptance below.

For a requested local tryout, build a final release candidate once, run applicable
CLI data-safety acceptance on that candidate, then package and perform
portable/native startup acceptance and installation in an agreed interactive
test window. Ordinary integration/commit work does not trigger that delivery.
Real ASIO hardware, other platforms and live provider APIs remain unverified for
this merge. Do not launch GUI/REAPER merely because the command is hidden.
Preserve libraries, Metadata and preferences.

## Final Windows local acceptance — 2026-11-17

Built release source `da031e493937f9d67080b0854c23574197a28739` once to
completion after resuming the interrupted build. License preflight, directory
packaging and portable acceptance PASS. No archive, tag or GitHub Release was
requested. Previously passing source checks above were reused; pipeline source
checks are explicitly `not-requested`, not a fresh source-suite pass.

During the authorized interactive window, an isolated synthetic 12-second
48 kHz / 24-bit stereo WAV and owned library were used with muted WASAPI output.
After a waveform drag selection, Space started playback, paused it, and resumed
it. Local IPC confirmed `playing → paused → playing`, volume zero and an
unchanged frame range `[103085, 336764)`.

The user's modified REAPER project was preserved. The user saved/closed their
session before native sending. Only the persistent dedicated REAPER profile
was then opened, with Dummy Audio confirmed in the actual host UI. The enabled
Still Evaluating control was confirmed normally; no licensing state was changed.
No Lua was launched; the existing native bridge DLL handled both sends.

The no-selected-track guard rejected the first button attempt without inserting
media. After selecting the fixture's single track, the button inserted one
12-second item at edit cursor 2.5 seconds. Its imported WAV was byte-identical
to the source. With Auto-Focus enabled, S inserted the selected 233679 frames
(4.8683125 seconds) at 2.5 seconds and brought REAPER in front of AudioLib without
manual activation. The app's success status had no focus-denied/unavailable
suffix. The saved owned RPP contained exactly two items. The slice retained
48 kHz, 24 bits and stereo; its PCM bytes matched the exact selected source
frames. Both test applications were closed normally afterward.

Native execution generated an empty `.audiolib-update.lock` in the portable
directory. The installer correctly rejected that non-package file before
copying. After closing the test app, only that owned runtime lock was moved to
private test evidence; the unchanged accepted payload was installed successfully.
`install-local-release.ps1` verified 908 package-owned files, copied 31 changed
files, confirmed `AudioLibCli 1.0.2` and passed isolated installed-desktop startup.
The fixed installation is now `local_release/`; working libraries, Metadata and
preferences were not replaced. No full build or portable suite was repeated.

Accepted executable SHA256 identities:

- Desktop: `4379ECC2E0CF1598454E59704D444F371557077C9BBC44B99865DA8FF5510DE4`
- CLI: `A7DEC0DB2264B506CDFC78E86C8B5607C3C6CFC39D7C2556FB9B0D2703616556`

Private receipts: `.rust-validation/release-pipeline/delivery-1.0.2/state.json`,
`.rust-validation/night-ui/delivery-native-run.json` and
`.rust-validation/local-install/268c67755d184fd5bdab4fec84747d14/report.json`.
The earlier Windows symlink-privilege limitation in `validate-review-fixes.ps1`
and unavailable real audio/other-platform coverage remain as recorded above.

After acceptance and installation, the verified owned `target/` Intermediate
cleanup PASS removed about 2.56 logical GiB, retaining about 1.88 GiB of final
outputs/other preserved contents. SDKs, package, installation, working data and
test evidence were preserved. A sandboxed plan could not open the Git lease;
the same plan/run with lease access passed, with no active compiler terminated.
The next action is ordinary user acceptance through
`local_release/Start-AudioLibExplorer.cmd`; no additional build is required.
