//! The exe's icon and version info, as the game's (`crates/app/build.rs`,
//! ticket 0902): the preview was one of `trpg-app`'s programs until ticket
//! 1269 and had them. Without a resource compiler the step is skipped with
//! a warning.
//!
//! On the web (ticket 0767) the link leaves the web shell's JS functions
//! as imports, as the game's does (`crates/app/build.rs` says why it is
//! set here and not in `.cargo/config.toml`).

fn main() {
    embed_windows_resources();
    if std::env::var("CARGO_CFG_TARGET_ARCH").as_deref() == Ok("wasm32") {
        println!("cargo:rustc-link-arg=--allow-undefined");
    }
}

// engine-friction: engine#3049, moves (game 1229 deletes it)
#[cfg(windows)]
fn embed_windows_resources() {
    if std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default() != "windows" {
        return;
    }
    println!("cargo:rerun-if-changed=../../assets/icon/icon.ico");
    let mut res = winresource::WindowsResource::new();
    res.set_icon("../../assets/icon/icon.ico")
        .set("FileDescription", "Visions of Shuyi script preview")
        .set("ProductName", "Visions of Shuyi")
        .set("InternalName", "story-preview")
        .set("OriginalFilename", "story-preview.exe");
    if let Err(e) = res.compile() {
        println!("cargo:warning=no exe icon or version info: {e}");
    }
}

// engine-friction: engine#3049, moves (game 1229 deletes it)
#[cfg(not(windows))]
fn embed_windows_resources() {}
