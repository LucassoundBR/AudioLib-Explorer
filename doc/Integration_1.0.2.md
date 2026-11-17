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

## Delivery boundary and next action

This is a source integration checkpoint. No new daily installation, release tag,
GitHub Release or upstream push is implied. `local_release/` still contains the
previous accepted 1.0.0 package; it must not be reported as accepted 1.0.2.

For a requested local tryout, build a final release candidate once, run applicable
CLI data-safety acceptance on that candidate, then package and perform
portable/native startup acceptance and installation in an agreed interactive
test window. Ordinary integration/commit work does not trigger that delivery.
Native playback/REAPER Auto-Focus, real ASIO hardware, other platforms and live
provider APIs remain unverified for this merge. Do not launch GUI/REAPER merely
because the command is hidden. Preserve libraries, Metadata and preferences.
