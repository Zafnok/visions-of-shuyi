//! `quad-storage`/`sapp-jsutils` (web storage, ticket 0207) and the
//! engine's `gamepad.js` (controllers, ticket 0219), in the web shell
//! `cargo shuyi web` packages (Shuyi Engine's `crates/cli/web/`, whose README says
//! why a game's binary sets this; ticket 1117), are miniquad JS plugins:
//! their `extern "C"` functions are implemented in JS and loaded into the
//! page at runtime, not linked in. Without `--allow-undefined`,
//! rust-lld treats them as link errors instead of leaving them as wasm
//! imports. Set here (via `cargo:rustc-link-arg`, additive) rather than in
//! `.cargo/config.toml`'s `rustflags`: a `RUSTFLAGS` env var (as CI sets,
//! for `-D warnings`) replaces config-file `rustflags` wholesale rather
//! than merging with it, which would silently drop this flag.

fn main() {
    embed_windows_resources();
    if std::env::var("CARGO_CFG_TARGET_ARCH").as_deref() == Ok("wasm32") {
        println!("cargo:rustc-link-arg=--allow-undefined");
    }
}

/// The exe's icon and version info (ticket 0902). The GNU target's resource
/// compiler is `windres`, in MSYS2's mingw-w64 binutils. Without a compiler
/// the step is skipped with a warning; `release.yml` refuses such a package
/// (`scripts/check-release-package.sh`).
#[cfg(windows)]
fn embed_windows_resources() {
    let target = |name: &str| std::env::var(name).unwrap_or_default();
    if target("CARGO_CFG_TARGET_OS") != "windows" {
        return;
    }
    println!("cargo:rerun-if-changed=../../assets/icon/icon.ico");
    let mut res = winresource::WindowsResource::new();
    res.set_icon("../../assets/icon/icon.ico")
        .set("FileDescription", "Visions of Shuyi")
        .set("ProductName", "Visions of Shuyi")
        .set("InternalName", "visions-of-shuyi")
        .set("OriginalFilename", "visions-of-shuyi.exe");
    // FileVersion and ProductVersion come from the package version.
    if let Err(e) = res.compile() {
        println!("cargo:warning=no exe icon or version info: {e}");
    }
}

#[cfg(not(windows))]
fn embed_windows_resources() {}
