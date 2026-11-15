# Fast development checkpoints

## Iteration and delivery

Research the existing implementation and official interfaces, define one small
deliverable, test the affected layer, review and commit. These practices adapt
ECC search-first and Superpowers planning; no frameworks/hooks are installed.

A development checkpoint may have integration checks pending. Label them and
record source identity; do not describe a compiled packet test as a native GUI
pass. Build/package/portable acceptance/fixed installation remain required at
application delivery, not at each development commit.

## Preview

```powershell
cargo build -p audiolib-desktop --bin AudioLibExplorer --profile preview --locked
```

This command compiles only; it does not start the application. Output is under
`target/x86_64-pc-windows-msvc/preview/`. The opt-in profile inherits dev: no
optimization or LTO, incremental compilation, limited debug information and
256 codegen units. Dev safety assertions/overflow checks remain enabled. Slint
build.rs retains development UI metadata. The existing dev/test/release settings
are unchanged. Use matching ASIO features only when that behavior is affected.

The first use rebuilds dependencies for the new profile; a warm-cache GUI speedup
has not been measured. Preview is not suitable for runtime performance claims,
release identity/privacy checks or installation into `local_release/`.

Cargo profile semantics: https://doc.rust-lang.org/cargo/reference/profiles.html

## Isolated tests

```powershell
pwsh -NoProfile -File tools/check-fast.ps1
pwsh -NoProfile -File tools/check-fast.ps1 -Action Run
```

Default is a read-only plan. Run performs strict Clippy and the isolated test
target, capturing timings and requiring a positive passed-test count. Use the
same configured Cargo/Rustup environment as normal project checks.

The independent manifest under `tools/fast-tests` imports actual native import,
focus parser and companion source by path. It has no Slint, audio engine, database
or desktop dependency. No implementation is copied or substituted. Only the
unused GUI entrypoints in the companion module allow dead_code in this harness;
other compiler/Clippy warnings still fail. Existing Controller/UI tests remain
in the desktop target and are run when those interactions change.

The harness has its own lockfile and workspace so Cargo does not resolve the GUI
workspace for a protocol edit. Outputs reuse the owned root target directory and
the shared build lease/cache budget; no separate unmanaged build cache is created.
Tests use synthetic files/peers and pure native-packet checks, not live REAPER or
window activation. Do not run the standalone probe's interactive mode here.

## Selecting verification

- Protocol/native pure logic: isolated checks first; relevant consumers when
  contracts change, native acceptance when behavior depends on the host.
- Controller/UI bindings: affected desktop checks plus interaction acceptance.
- Renderer/DSP/storage: relevant existing module regressions, preserving actual
  data-safety coverage. A mock or protocol test does not replace these checks.
- Delivery: complete the repository delivery/install sequence once for the final
  payload. Keep preview/development receipts distinct from release acceptance.

Keep caches within budget rather than deleting all debug output between normal
iterations. Changes to toolchain, target, features, profile or compiler flags can
require rebuilding. Never add check + Clippy + build passes automatically when
one already supplies the same compilation evidence.

## Recorded validation: 2026-11-15

Windows x64, Rust1.97.1 MSVC, no GUI/audio/input:

- Core-library preview: initial26.16s, warm0.63s. Verbose compiler invocation
  confirms lto=off, codegen-units256, debuginfo1 and incremental output. These
  are core-library measurements, not full GUI-preview speed claims.
- Isolated target: strict Clippy2.29s +8 tests3.11s on first harness use with
  existing project caches. Warm Clippy0.29s +8 tests0.26s (about0.55s Cargo time;
  shell startup adds overhead). No equivalent whole-desktop speedup is claimed:
  Controller/UI tests remain separate and were not executed in this tooling task.
- Windows dependency metadata:8 packages, no Slint/desktop/CPAL/rusqlite. Dep-info
  tracks actual native/focus/companion source and the embedded Lua template.
  Initial unfiltered offline metadata inspection failed on an uncached Unix
  errno dependency; using the actual Windows platform resolved the metadata.
  Cross-platform harness execution is not claimed.
- PowerShell syntax and read-only plan passed; Cargo formatting passed for both
  workspaces. Preview-cache functional regression passed in a verified synthetic
  subtree: plan does not delete, debug/host+target preview are removed when
  requested, release and outside-cache data sentinel remain unchanged.

Logs/receipts are under ignored `.rust-validation/fast-development/` and
`.rust-validation/fast-checks/`. Neither release source behavior nor shipped
features changed. The existing native REAPER candidate from35c393b remains
pending GUI acceptance/installation; this workflow change does not rebuild it
or install preview. The shared-computer no-interface restriction remains active.
Final cache maintenance passed: 22,797,267,921 logical bytes, below 40 GiB; zero bytes removed. Preview is included in development-cache accounting.
