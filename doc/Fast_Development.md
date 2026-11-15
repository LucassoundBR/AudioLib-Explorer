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


## Preview validation

Rust 1.97.1 MSVC core-library preview build passed: 26.16s initial, 0.63s warm.
Actual compiler flags confirm no LTO, limited debug info and incremental output.
This measures the core library, not a full GUI build. No interface was launched.

The managed development cache budget now includes debug and preview. Three
functional checks in tools/test-preview-cache.ps1 passed in an isolated fixture:
read-only plan, removal of debug/host+target preview, preservation of release
and an outside-cache user-data sentinel. Release profile is unchanged.
