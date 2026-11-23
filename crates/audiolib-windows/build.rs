// crates/audiolib-windows/build.rs
fn main() {
    println!("cargo:rerun-if-changed=src/reaper_extension_host.rs");
    println!("cargo:rerun-if-changed=src/reaper_protocol.rs");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        let output = std::path::PathBuf::from(std::env::var_os("OUT_DIR").unwrap())
            .join("reaper_audiolib_bridge.dll");
        let status = std::process::Command::new(std::env::var_os("RUSTC").unwrap())
            .args([
                "--edition=2024",
                "--crate-type=cdylib",
                "--crate-name=audiolib_reaper_extension",
                "--target",
            ])
            .arg(std::env::var("TARGET").unwrap())
            .args([
                "-Copt-level=2",
                "-Cdebuginfo=0",
                "-Cstrip=symbols",
                "-Cpanic=abort",
                "-Ctarget-feature=+crt-static",
                "src/reaper_extension_host.rs",
                "-o",
            ])
            .arg(output)
            .status()
            .expect("Run rustc for the native REAPER extension");
        assert!(
            status.success(),
            "Native REAPER extension compilation failed"
        );
    }
    println!("cargo:rerun-if-env-changed=CPAL_ASIO_DIR");
    println!("cargo:rerun-if-changed=src/asio_shim.cpp");
    if std::env::var_os("CARGO_FEATURE_ASIO").is_some()
        && std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows")
    {
        let sdk = std::env::var_os("CPAL_ASIO_DIR")
            .expect("Set CPAL_ASIO_DIR to the approved ASIO SDK path");
        cc::Build::new()
            .cpp(true)
            .file("src/asio_shim.cpp")
            .include(std::path::PathBuf::from(sdk).join("common"))
            .compile("audiolib_asio_shim");
    }
}
