//! `cargo xtask playdate [--release] [--device|--simulator|--both]`: builds
//! the Playdate front end (`crates/playdate`, ADR-0070) into
//! `target/playdate/VisionsOfShuyi.pdx` (ticket 1031).
//!
//! For each target it builds the Rust static library (`trpg-playdate` with
//! its `sdk` feature), compiles and links the C side (`shim/shim.c` and the
//! SDK's `setup.c`) with the flags of the SDK's `buildsupport/common.mk`,
//! then packs both with the SDK's `pdc`:
//!
//! - the device: `thumbv7em-none-eabihf` and `arm-none-eabi-gcc`, linked
//!   with the SDK's `link_map.ld` into `pdex.elf`;
//! - the Simulator (Windows only): `x86_64-pc-windows-gnu` and MinGW
//!   `gcc`, linked into `pdex.dll`.
//!
//! The default is both. `--release` builds with the `playdate` profile
//! (what goes on a device), otherwise `playdate-dev`. The SDK comes from
//! `PLAYDATE_SDK_PATH`, else `~/Documents/PlaydateSDK`; the compilers from
//! `PATH`, else MSYS2's default places. GNU tools only: no MSVC, no
//! nightly, no `CMake`.
//!
//! The build is a list of [`Step`]s ([`plan`]), so the tests check every
//! command without the SDK; [`execute`] runs them.

use std::env;
use std::ffi::{OsStr, OsString};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// The game's name on the Playdate (`pdxinfo`). The SDK's license keeps
/// "Playdate" and "Panic" out of it (ADR-0070).
const GAME_NAME: &str = "Visions of Shuyi";

/// The `author` in `pdxinfo`.
const AUTHOR: &str = "Nick Wentz";

/// The Playdate's bundle ID: the game's data folder on the device is named
/// after it, so it never changes once a build is out.
pub const BUNDLE_ID: &str = "com.zafnok.visionsofshuyi";

/// The packed game, in `target/playdate/`.
const PDX_NAME: &str = "VisionsOfShuyi.pdx";

/// The device's Rust target (ADR-0070 §1).
const DEVICE_TARGET: &str = "thumbv7em-none-eabihf";

/// The Simulator's Rust target: MinGW, like Nick's host toolchain.
const SIMULATOR_TARGET: &str = "x86_64-pc-windows-gnu";

/// The device's Rust flags (ADR-0070 §1): Rev A's single-precision FPU,
/// and position-independent code, so no MOVW/MOVT address pairs, which
/// `pdc` cannot relocate.
const DEVICE_RUSTFLAGS: &str =
    "-C target-cpu=cortex-m7 -C target-feature=-fp64 -C relocation-model=pic";

/// The C compiler's CPU flags for the device (`common.mk`'s `MCFLAGS`).
const DEVICE_CPU_FLAGS: &[&str] = &[
    "-mthumb",
    "-mcpu=cortex-m7",
    "-mfloat-abi=hard",
    "-mfpu=fpv5-sp-d16",
    "-D__FPU_USED=1",
];

/// The rest of `common.mk`'s compile flags for the device (`OPT`,
/// `CPFLAGS`, `DDEFS`), without its listing and dependency files.
const DEVICE_COMPILE_FLAGS: &[&str] = &[
    "-O2",
    "-falign-functions=16",
    "-fomit-frame-pointer",
    "-g3",
    "-gdwarf-2",
    "-Wall",
    "-Wno-unused",
    "-Wstrict-prototypes",
    "-Wno-unknown-pragmas",
    "-Wdouble-promotion",
    "-mword-relocations",
    "-fno-common",
    "-ffunction-sections",
    "-fdata-sections",
    "-DTARGET_PLAYDATE=1",
    "-DTARGET_EXTENSION=1",
];

/// Where MSYS2 puts the device's compiler
/// (`mingw-w64-ucrt-x86_64-arm-none-eabi-toolchain`).
const MSYS2_ARM_GCC: &str = r"C:\msys64\ucrt64\bin\arm-none-eabi-gcc.exe";

/// Where MSYS2 puts MinGW's compiler (`mingw-w64-x86_64-gcc`), the one
/// `.cargo/config.toml` links the game with.
const MSYS2_GCC: &str = r"C:\msys64\mingw64\bin\gcc.exe";

/// Usage line for bad arguments.
const USAGE: &str = "usage: cargo xtask playdate [--release] [--device|--simulator|--both]";

/// Parsed `cargo xtask playdate` arguments.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Options {
    /// Build with the `playdate` profile instead of `playdate-dev`.
    pub release: bool,
    /// Build `pdex.elf`, for the device.
    pub device: bool,
    /// Build `pdex.dll`, for the Simulator.
    pub simulator: bool,
}

/// Parses the arguments after `playdate`: `--release`, and at most one of
/// `--device`, `--simulator` and `--both` (the default), each at most once.
pub fn parse_args(args: &[String]) -> Result<Options, String> {
    let mut release = false;
    let mut which: Option<(bool, bool)> = None;
    for arg in args {
        let targets = match arg.as_str() {
            "--release" if !release => {
                release = true;
                continue;
            }
            "--device" => (true, false),
            "--simulator" => (false, true),
            "--both" => (true, true),
            _ => return Err(USAGE.to_string()),
        };
        if which.replace(targets).is_some() {
            return Err(USAGE.to_string());
        }
    }
    let (device, simulator) = which.unwrap_or((true, true));
    Ok(Options {
        release,
        device,
        simulator,
    })
}

/// Cargo's profile for the build: `playdate` with `--release`.
fn profile(release: bool) -> &'static str {
    if release { "playdate" } else { "playdate-dev" }
}

/// The Rust static library cargo builds for `target`.
fn rust_lib(root: &Path, target: &str, release: bool) -> PathBuf {
    root.join("target")
        .join(target)
        .join(profile(release))
        .join("libtrpg_playdate.a")
}

/// `RUSTFLAGS` for the device build: the caller's own (CI's
/// `-D warnings`) and then the device's, because `RUSTFLAGS` replaces
/// every other place Rust flags can come from.
fn device_rustflags(existing: Option<&str>) -> String {
    match existing.map(str::trim).filter(|flags| !flags.is_empty()) {
        Some(flags) => format!("{flags} {DEVICE_RUSTFLAGS}"),
        None => DEVICE_RUSTFLAGS.to_string(),
    }
}

/// The `cargo` arguments that build the static library for `target`.
fn cargo_args(target: &str, release: bool) -> Vec<OsString> {
    [
        "build",
        "-p",
        "trpg-playdate",
        "--lib",
        "--features",
        "sdk",
        "--target",
        target,
        "--profile",
        profile(release),
    ]
    .into_iter()
    .map(OsString::from)
    .collect()
}

/// The `pdxinfo` file: the game's name, author, bundle ID and version
/// (`version` is the workspace's, `x.y.z`).
pub fn pdxinfo(version: &str) -> Result<String, String> {
    let build = build_number(version)?;
    Ok(format!(
        "name={GAME_NAME}\nauthor={AUTHOR}\nbundleID={BUNDLE_ID}\nversion={version}\nbuildNumber={build}\n"
    ))
}

/// The `buildNumber` for version `x.y.z`: `x * 10000 + y * 100 + z`, so it
/// only ever goes up, as the Playdate asks.
pub fn build_number(version: &str) -> Result<u32, String> {
    let parts: Vec<u32> = version
        .split('.')
        .map(str::parse)
        .collect::<Result<_, _>>()
        .map_err(|_| format!("version `{version}` is not x.y.z"))?;
    match parts[..] {
        [major, minor, patch] if minor < 100 && patch < 100 => {
            Ok(major * 10_000 + minor * 100 + patch)
        }
        _ => Err(format!(
            "version `{version}` is not x.y.z with y and z under 100"
        )),
    }
}

/// Where to look for the SDK, in order: `PLAYDATE_SDK_PATH`, then
/// `~/Documents/PlaydateSDK` (the installer's default).
pub fn sdk_candidates(sdk_path: Option<OsString>, home: Option<PathBuf>) -> Vec<PathBuf> {
    let from_env = sdk_path.filter(|path| !path.is_empty()).map(PathBuf::from);
    let default = home.map(|home| home.join("Documents").join("PlaydateSDK"));
    from_env.into_iter().chain(default).collect()
}

/// The first of `candidates` that holds an SDK (its `C_API/pd_api.h`).
pub fn find_sdk(candidates: &[PathBuf]) -> Result<PathBuf, String> {
    if let Some(sdk) = candidates
        .iter()
        .find(|path| path.join("C_API").join("pd_api.h").is_file())
    {
        return Ok(sdk.clone());
    }
    let looked: Vec<String> = candidates.iter().map(|p| p.display().to_string()).collect();
    Err(format!(
        "the Playdate SDK was not found (looked in: {}). Install SDK 3.1.2 from \
         https://play.date/dev/ and set PLAYDATE_SDK_PATH if it is not in ~/Documents/PlaydateSDK",
        if looked.is_empty() {
            "nowhere: no PLAYDATE_SDK_PATH and no home folder".to_string()
        } else {
            looked.join(", ")
        }
    ))
}

/// The program `file_name` (with `.exe` on Windows) in a folder of
/// `path_var` (`PATH`), else the first of `fallbacks` that exists.
pub fn find_tool(
    file_name: &str,
    path_var: Option<&OsStr>,
    fallbacks: &[&Path],
) -> Option<PathBuf> {
    path_var
        .into_iter()
        .flat_map(env::split_paths)
        .map(|dir| dir.join(file_name))
        .chain(fallbacks.iter().map(|path| path.to_path_buf()))
        .find(|path| path.is_file())
}

/// Whether the Rust toolchain at `sysroot` has the standard library for
/// `target` (`rustup target add <target>`).
pub fn target_installed(sysroot: &Path, target: &str) -> bool {
    sysroot
        .join("lib")
        .join("rustlib")
        .join(target)
        .join("lib")
        .is_dir()
}

/// What the build needs from the machine.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tools {
    /// The Playdate SDK.
    pub sdk: PathBuf,
    /// `arm-none-eabi-gcc`, if found.
    pub arm_gcc: Option<PathBuf>,
    /// MinGW's `gcc`, if found.
    pub gcc: Option<PathBuf>,
    /// Whether each Rust target is installed: the device's, the Simulator's.
    pub targets: (bool, bool),
    /// Whether this is Windows (the Simulator build needs it).
    pub windows: bool,
    /// The `RUSTFLAGS` the build was started with.
    pub rustflags: Option<String>,
}

/// One thing the build does.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Step {
    /// Makes `path` an empty folder.
    Fresh(PathBuf),
    /// Runs `program` with `args`; `RUSTFLAGS` set to `rustflags` if any.
    Run {
        /// What it does, for the error when it fails.
        what: &'static str,
        /// The program.
        program: PathBuf,
        /// Its arguments.
        args: Vec<OsString>,
        /// `RUSTFLAGS` for it.
        rustflags: Option<String>,
    },
    /// Writes `text` to `path`.
    Write {
        /// The file.
        path: PathBuf,
        /// Its text.
        text: String,
    },
}

/// The build's output folder, `target/playdate/`.
pub fn out_dir(root: &Path) -> PathBuf {
    root.join("target").join("playdate")
}

/// `items` as arguments.
fn os_args<I, S>(items: I) -> Vec<OsString>
where
    I: IntoIterator<Item = S>,
    S: Into<OsString>,
{
    items.into_iter().map(Into::into).collect()
}

/// The files and folders a build reads and writes.
struct Paths {
    /// The workspace.
    root: PathBuf,
    /// `target/playdate/`.
    out: PathBuf,
    /// What `pdc` packs: `pdex.elf`, `pdex.dll`, `pdxinfo`.
    source: PathBuf,
    /// The C side's object files and the device's link map.
    obj: PathBuf,
    /// Our C shim.
    shim: PathBuf,
    /// The SDK's C headers.
    c_api: PathBuf,
    /// The SDK's `setup.c`.
    setup: PathBuf,
}

impl Paths {
    fn new(root: &Path, sdk: &Path) -> Self {
        let out = out_dir(root);
        let c_api = sdk.join("C_API");
        Self {
            root: root.to_path_buf(),
            source: out.join("Source"),
            obj: out.join("obj"),
            out,
            shim: root.join("crates/playdate/shim/shim.c"),
            setup: c_api.join("buildsupport").join("setup.c"),
            c_api,
        }
    }
}

/// The steps that build the `.pdx` for `options` with `tools`, the
/// workspace at `root` and its version `version`.
pub fn plan(
    root: &Path,
    options: Options,
    tools: &Tools,
    version: &str,
) -> Result<Vec<Step>, String> {
    let paths = Paths::new(root, &tools.sdk);
    let mut steps = vec![
        Step::Fresh(paths.source.clone()),
        Step::Fresh(paths.obj.clone()),
    ];
    if options.device {
        steps.extend(device_steps(&paths, options.release, tools)?);
    }
    if options.simulator {
        steps.extend(simulator_steps(&paths, options.release, tools)?);
    }
    steps.push(Step::Write {
        path: paths.source.join("pdxinfo"),
        text: pdxinfo(version)?,
    });
    let sdk = &tools.sdk;
    let pdc = sdk
        .join("bin")
        .join(format!("pdc{}", env::consts::EXE_SUFFIX));
    steps.push(Step::Run {
        what: "pdc",
        program: pdc,
        args: os_args([
            OsString::from("-sdkpath"),
            sdk.clone().into_os_string(),
            paths.source.into_os_string(),
            paths.out.join(PDX_NAME).into_os_string(),
        ]),
        rustflags: None,
    });
    Ok(steps)
}

/// The steps that build `pdex.elf`, for the device.
fn device_steps(paths: &Paths, release: bool, tools: &Tools) -> Result<Vec<Step>, String> {
    if !tools.targets.0 {
        return Err(format!(
            "the Rust target {DEVICE_TARGET} is not installed: rustup target add {DEVICE_TARGET}"
        ));
    }
    let Some(arm_gcc) = tools.arm_gcc.clone() else {
        return Err(format!(
            "arm-none-eabi-gcc was not found on PATH or at {MSYS2_ARM_GCC}. Install it with              MSYS2 (pacman -S mingw-w64-ucrt-x86_64-arm-none-eabi-toolchain) or, on Linux,              apt install gcc-arm-none-eabi"
        ));
    };
    let compile = |input: &Path, output: PathBuf, strict: bool| {
        let mut args = os_args(DEVICE_CPU_FLAGS.iter().chain(DEVICE_COMPILE_FLAGS).copied());
        if strict {
            args.push("-Werror".into());
        }
        args.extend(os_args(["-I".into(), paths.c_api.clone().into_os_string()]));
        args.extend(os_args(["-c".into(), input.as_os_str().to_owned()]));
        args.extend(os_args(["-o".into(), output.into_os_string()]));
        args
    };
    let mut link = os_args([
        paths.obj.join("shim.o"),
        paths.obj.join("setup.o"),
        rust_lib(&paths.root, DEVICE_TARGET, release),
    ]);
    link.push("-nostartfiles".into());
    link.extend(os_args(DEVICE_CPU_FLAGS.iter().copied()));
    let mut script = OsString::from("-T");
    script.push(paths.c_api.join("buildsupport").join("link_map.ld"));
    link.push(script);
    let mut map = OsString::from("-Wl,-Map=");
    map.push(paths.obj.join("pdex.map"));
    map.push(",--cref,--gc-sections,--no-warn-mismatch,--emit-relocs");
    link.push(map);
    link.extend(os_args([
        "-o".into(),
        paths.source.join("pdex.elf").into_os_string(),
    ]));
    Ok(vec![
        Step::Run {
            what: "cargo build (device)",
            program: PathBuf::from("cargo"),
            args: cargo_args(DEVICE_TARGET, release),
            rustflags: Some(device_rustflags(tools.rustflags.as_deref())),
        },
        Step::Run {
            what: "compile shim.c (device)",
            program: arm_gcc.clone(),
            // Our own C: no warnings.
            args: compile(&paths.shim, paths.obj.join("shim.o"), true),
            rustflags: None,
        },
        Step::Run {
            what: "compile setup.c (device)",
            program: arm_gcc.clone(),
            args: compile(&paths.setup, paths.obj.join("setup.o"), false),
            rustflags: None,
        },
        Step::Run {
            what: "link pdex.elf (device)",
            program: arm_gcc,
            args: link,
            rustflags: None,
        },
    ])
}

/// The steps that build `pdex.dll`, for the Simulator.
fn simulator_steps(paths: &Paths, release: bool, tools: &Tools) -> Result<Vec<Step>, String> {
    if !tools.windows {
        return Err(
            "the Simulator build (pdex.dll) is made on Windows only; pass --device".to_string(),
        );
    }
    if !tools.targets.1 {
        return Err(format!(
            "the Rust target {SIMULATOR_TARGET} is not installed: rustup target add {SIMULATOR_TARGET}"
        ));
    }
    let Some(gcc) = tools.gcc.clone() else {
        return Err(format!(
            "MinGW gcc was not found on PATH or at {MSYS2_GCC}. Install it with MSYS2              (pacman -S mingw-w64-x86_64-gcc)"
        ));
    };
    let mut args = os_args([
        "-shared",
        "-O2",
        // Stripped: the `.pdx` carries the Simulator's file to the device too.
        "-s",
        "-Wall",
        "-D_WINDLL",
        "-DTARGET_SIMULATOR=1",
        "-DTARGET_EXTENSION=1",
        "-I",
    ]);
    args.push(paths.c_api.clone().into_os_string());
    args.extend(os_args([
        paths.shim.clone(),
        paths.setup.clone(),
        rust_lib(&paths.root, SIMULATOR_TARGET, release),
    ]));
    args.extend(os_args(["-static-libgcc", "-o"]));
    args.push(paths.source.join("pdex.dll").into_os_string());
    Ok(vec![
        Step::Run {
            what: "cargo build (Simulator)",
            program: PathBuf::from("cargo"),
            args: cargo_args(SIMULATOR_TARGET, release),
            rustflags: None,
        },
        Step::Run {
            what: "link pdex.dll (Simulator)",
            program: gcc,
            args,
            rustflags: None,
        },
    ])
}

/// Runs `steps` in order, stopping at the first that fails.
pub fn execute(steps: &[Step]) -> Result<(), String> {
    for step in steps {
        match step {
            Step::Fresh(path) => {
                if path.exists() {
                    fs::remove_dir_all(path)
                        .map_err(|e| format!("clear {}: {e}", path.display()))?;
                }
                fs::create_dir_all(path).map_err(|e| format!("create {}: {e}", path.display()))?;
            }
            Step::Run {
                what,
                program,
                args,
                rustflags,
            } => {
                let mut command = Command::new(program);
                command.args(args);
                if let Some(flags) = rustflags {
                    command.env("RUSTFLAGS", flags);
                }
                let status = command
                    .status()
                    .map_err(|e| format!("{what}: could not start {}: {e}", program.display()))?;
                if !status.success() {
                    return Err(format!("{what} failed ({status}): {command:?}"));
                }
            }
            Step::Write { path, text } => {
                fs::write(path, text).map_err(|e| format!("write {}: {e}", path.display()))?;
            }
        }
    }
    Ok(())
}

/// A line naming the `.pdx` and the size of what went into it.
pub fn summary(root: &Path, options: Options) -> String {
    let out = out_dir(root);
    let size = |name: &str| {
        fs::metadata(out.join("Source").join(name)).map_or_else(
            |_| format!("{name} missing"),
            |meta| format!("{name} {} bytes", meta.len()),
        )
    };
    let mut parts = Vec::new();
    if options.device {
        parts.push(size("pdex.elf"));
    }
    if options.simulator {
        parts.push(size("pdex.dll"));
    }
    format!("{} ({})", out.join(PDX_NAME).display(), parts.join(", "))
}

/// The Rust toolchain's sysroot (`rustc --print sysroot`).
fn sysroot() -> Option<PathBuf> {
    let output = Command::new("rustc")
        .args(["--print", "sysroot"])
        .output()
        .ok()?;
    output
        .status
        .success()
        .then(|| PathBuf::from(String::from_utf8_lossy(&output.stdout).trim()))
}

/// Finds the SDK and tools on this machine.
pub fn find_tools() -> Result<Tools, String> {
    let home = env::var_os(if cfg!(windows) { "USERPROFILE" } else { "HOME" }).map(PathBuf::from);
    let sdk = find_sdk(&sdk_candidates(env::var_os("PLAYDATE_SDK_PATH"), home))?;
    let path = env::var_os("PATH");
    let exe = |name: &str| format!("{name}{}", env::consts::EXE_SUFFIX);
    let msys2 = |path: &'static str| {
        if cfg!(windows) {
            vec![Path::new(path)]
        } else {
            vec![]
        }
    };
    let sysroot = sysroot().unwrap_or_default();
    Ok(Tools {
        sdk,
        arm_gcc: find_tool(
            &exe("arm-none-eabi-gcc"),
            path.as_deref(),
            &msys2(MSYS2_ARM_GCC),
        ),
        gcc: find_tool(&exe("gcc"), path.as_deref(), &msys2(MSYS2_GCC)),
        targets: (
            target_installed(&sysroot, DEVICE_TARGET),
            target_installed(&sysroot, SIMULATOR_TARGET),
        ),
        windows: cfg!(windows),
        rustflags: env::var("RUSTFLAGS").ok(),
    })
}

/// Builds the `.pdx`, returning a one-line summary on success.
///
/// Not covered by mutation testing (`#[mutants::skip]`): it only joins
/// [`find_tools`], [`plan`] and [`execute`] (each tested), and running it
/// needs the Playdate SDK and the compilers; CI's `playdate` job runs it.
#[mutants::skip]
pub fn run(root: &Path, options: Options) -> Result<String, String> {
    let tools = find_tools()?;
    let steps = plan(root, options, &tools, env!("CARGO_PKG_VERSION"))?;
    execute(&steps)?;
    Ok(summary(root, options))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(items: &[&str]) -> Vec<String> {
        items.iter().map(|s| (*s).to_string()).collect()
    }

    /// A scratch folder under the system's temp folder, empty.
    fn scratch(name: &str) -> PathBuf {
        let dir = env::temp_dir().join(format!("xtask-playdate-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn tools() -> Tools {
        Tools {
            sdk: PathBuf::from("/sdk"),
            arm_gcc: Some(PathBuf::from("/bin/arm-none-eabi-gcc")),
            gcc: Some(PathBuf::from("/mingw/gcc")),
            targets: (true, true),
            windows: true,
            rustflags: None,
        }
    }

    const BOTH: Options = Options {
        release: true,
        device: true,
        simulator: true,
    };

    /// The steps that run `program`, with their arguments as strings.
    fn runs(steps: &[Step]) -> Vec<(&'static str, String, Vec<String>, Option<String>)> {
        steps
            .iter()
            .filter_map(|step| match step {
                Step::Run {
                    what,
                    program,
                    args,
                    rustflags,
                } => Some((
                    *what,
                    program.display().to_string(),
                    args.iter()
                        .map(|a| a.to_string_lossy().replace('\\', "/"))
                        .collect(),
                    rustflags.clone(),
                )),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn the_default_is_both_targets_without_release() {
        assert_eq!(
            parse_args(&[]),
            Ok(Options {
                release: false,
                device: true,
                simulator: true
            })
        );
    }

    #[test]
    fn each_flag_picks_its_targets() {
        let parsed = |a: &[&str]| parse_args(&args(a)).unwrap();
        assert_eq!(
            parsed(&["--release"]),
            Options {
                release: true,
                ..parse_args(&[]).unwrap()
            }
        );
        let device = parsed(&["--device", "--release"]);
        assert!(device.release && device.device && !device.simulator);
        let simulator = parsed(&["--simulator"]);
        assert!(!simulator.release && !simulator.device && simulator.simulator);
        assert_eq!(parsed(&["--both", "--release"]), BOTH);
    }

    #[test]
    fn bad_or_repeated_flags_are_refused() {
        for bad in [
            &["--bogus"][..],
            &["--release", "--release"],
            &["--device", "--simulator"],
            &["--both", "--both"],
            &["--device", "--device"],
        ] {
            assert_eq!(parse_args(&args(bad)), Err(USAGE.to_string()), "{bad:?}");
        }
    }

    #[test]
    fn pdxinfo_names_the_game_and_its_version() {
        assert_eq!(
            pdxinfo("0.2.0").unwrap(),
            "name=Visions of Shuyi\nauthor=Nick Wentz\nbundleID=com.zafnok.visionsofshuyi\n\
             version=0.2.0\nbuildNumber=200\n"
        );
    }

    #[test]
    fn the_build_number_goes_up_with_the_version() {
        assert_eq!(build_number("0.2.0"), Ok(200));
        assert_eq!(build_number("0.2.1"), Ok(201));
        assert_eq!(build_number("1.0.0"), Ok(10_000));
        assert_eq!(build_number("1.99.99"), Ok(19_999));
        assert!(build_number("1.100.0").is_err());
        assert!(build_number("1.0.100").is_err());
        assert!(build_number("1.0").is_err());
        assert!(build_number("1.0.0.0").is_err());
        assert!(build_number("1.x.0").is_err());
        assert!(pdxinfo("1.0").is_err());
    }

    #[test]
    fn the_sdk_is_looked_for_in_the_variable_then_the_documents_folder() {
        let home = PathBuf::from("/home/nick");
        assert_eq!(
            sdk_candidates(Some("/opt/sdk".into()), Some(home.clone())),
            [
                PathBuf::from("/opt/sdk"),
                home.join("Documents").join("PlaydateSDK")
            ]
        );
        assert_eq!(
            sdk_candidates(Some(OsString::new()), Some(home.clone())),
            [home.join("Documents").join("PlaydateSDK")]
        );
        assert_eq!(sdk_candidates(None, None), Vec::<PathBuf>::new());
    }

    #[test]
    fn the_first_folder_with_the_sdks_header_is_the_sdk() {
        let dir = scratch("sdk");
        let empty = dir.join("empty");
        let sdk = dir.join("sdk");
        fs::create_dir_all(&empty).unwrap();
        fs::create_dir_all(sdk.join("C_API")).unwrap();
        fs::write(sdk.join("C_API").join("pd_api.h"), "").unwrap();
        assert_eq!(find_sdk(&[empty.clone(), sdk.clone()]), Ok(sdk));
        let missing = find_sdk(&[empty]).unwrap_err();
        assert!(
            missing.contains("PLAYDATE_SDK_PATH") && missing.contains("3.1.2"),
            "{missing}"
        );
        assert!(missing.contains("empty"), "{missing}");
        assert!(find_sdk(&[]).unwrap_err().contains("nowhere"));
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn a_tool_on_path_comes_before_the_fallback() {
        let dir = scratch("tools");
        let (first, second) = (dir.join("first"), dir.join("second"));
        fs::create_dir_all(&first).unwrap();
        fs::create_dir_all(&second).unwrap();
        fs::write(second.join("cc"), "").unwrap();
        let fallback = dir.join("fallback-cc");
        fs::write(&fallback, "").unwrap();
        let path = env::join_paths([&first, &second]).unwrap();
        assert_eq!(
            find_tool("cc", Some(&path), &[&fallback]),
            Some(second.join("cc"))
        );
        assert_eq!(
            find_tool("other", Some(&path), &[&fallback]),
            Some(fallback.clone())
        );
        assert_eq!(find_tool("other", None, &[&dir.join("nope")]), None);
        // A folder of that name is not the tool.
        fs::create_dir_all(first.join("dir-cc")).unwrap();
        assert_eq!(find_tool("dir-cc", Some(&path), &[]), None);
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn a_target_is_installed_when_its_library_folder_is_in_the_sysroot() {
        let dir = scratch("sysroot");
        fs::create_dir_all(dir.join("lib/rustlib").join(DEVICE_TARGET).join("lib")).unwrap();
        assert!(target_installed(&dir, DEVICE_TARGET));
        assert!(!target_installed(&dir, SIMULATOR_TARGET));
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn the_sysroot_is_the_running_toolchains() {
        // The toolchain that runs the tests has its own standard library.
        let root = sysroot().unwrap();
        assert!(
            root.join("lib").join("rustlib").is_dir(),
            "{}",
            root.display()
        );
    }

    #[test]
    fn the_device_rustflags_follow_the_callers() {
        assert_eq!(device_rustflags(None), DEVICE_RUSTFLAGS);
        assert_eq!(device_rustflags(Some("  ")), DEVICE_RUSTFLAGS);
        assert_eq!(
            device_rustflags(Some("-D warnings")),
            format!("-D warnings {DEVICE_RUSTFLAGS}")
        );
        assert!(DEVICE_RUSTFLAGS.contains("target-cpu=cortex-m7"));
        assert!(DEVICE_RUSTFLAGS.contains("target-feature=-fp64"));
        assert!(DEVICE_RUSTFLAGS.contains("relocation-model=pic"));
    }

    #[test]
    fn both_targets_build_the_libraries_link_them_and_pack_one_pdx() {
        let root = Path::new("/repo");
        let steps = plan(root, BOTH, &tools(), "0.2.0").unwrap();
        let out = root.join("target").join("playdate");
        assert_eq!(steps[0], Step::Fresh(out.join("Source")));
        assert_eq!(steps[1], Step::Fresh(out.join("obj")));
        let whats: Vec<&str> = runs(&steps).iter().map(|r| r.0).collect();
        assert_eq!(
            whats,
            [
                "cargo build (device)",
                "compile shim.c (device)",
                "compile setup.c (device)",
                "link pdex.elf (device)",
                "cargo build (Simulator)",
                "link pdex.dll (Simulator)",
                "pdc",
            ]
        );
        assert!(steps.contains(&Step::Write {
            path: out.join("Source").join("pdxinfo"),
            text: pdxinfo("0.2.0").unwrap(),
        }));
        // pdxinfo is written before pdc packs the folder.
        let pdxinfo_at = steps.iter().position(|s| matches!(s, Step::Write { .. }));
        assert_eq!(pdxinfo_at, Some(steps.len() - 2));
    }

    #[test]
    fn the_rust_builds_use_the_playdate_profiles_and_the_sdk_feature() {
        let built = runs(&plan(Path::new("/repo"), BOTH, &tools(), "0.2.0").unwrap());
        let device = &built[0];
        assert_eq!(device.1, "cargo");
        assert_eq!(
            device.2.join(" "),
            "build -p trpg-playdate --lib --features sdk --target thumbv7em-none-eabihf --profile playdate"
        );
        assert_eq!(device.3.as_deref(), Some(DEVICE_RUSTFLAGS));
        let simulator = &built[4];
        assert_eq!(
            simulator.2.join(" "),
            "build -p trpg-playdate --lib --features sdk --target x86_64-pc-windows-gnu --profile playdate"
        );
        // The Simulator is a PC build: no device flags.
        assert_eq!(simulator.3, None);

        let debug = Options {
            release: false,
            ..BOTH
        };
        let built = runs(&plan(Path::new("/repo"), debug, &tools(), "0.2.0").unwrap());
        assert_eq!(
            built[0].2[built[0].2.len() - 2..],
            ["--profile", "playdate-dev"]
        );
        assert_eq!(
            built[3].2[2],
            "/repo/target/thumbv7em-none-eabihf/playdate-dev/libtrpg_playdate.a"
        );
    }

    #[test]
    fn the_callers_rustflags_reach_the_device_build() {
        let tools = Tools {
            rustflags: Some("-D warnings".into()),
            ..tools()
        };
        let steps = plan(Path::new("/repo"), BOTH, &tools, "0.2.0").unwrap();
        assert_eq!(
            runs(&steps)[0].3,
            Some(format!("-D warnings {DEVICE_RUSTFLAGS}"))
        );
    }

    #[test]
    fn the_device_c_side_uses_the_sdks_flags() {
        let built = runs(&plan(Path::new("/repo"), BOTH, &tools(), "0.2.0").unwrap());
        let shim = &built[1];
        assert_eq!(shim.1.replace('\\', "/"), "/bin/arm-none-eabi-gcc");
        for flag in [
            "-mthumb",
            "-mcpu=cortex-m7",
            "-mfloat-abi=hard",
            "-mfpu=fpv5-sp-d16",
            "-mword-relocations",
            "-DTARGET_PLAYDATE=1",
            "-DTARGET_EXTENSION=1",
            "-Werror",
        ] {
            assert!(shim.2.contains(&flag.to_string()), "{flag}: {:?}", shim.2);
        }
        let tail = &shim.2[shim.2.len() - 6..];
        assert_eq!(
            tail,
            [
                "-I",
                "/sdk/C_API",
                "-c",
                "/repo/crates/playdate/shim/shim.c",
                "-o",
                "/repo/target/playdate/obj/shim.o"
            ]
        );
        // The SDK's own file is not held to our warnings.
        let setup = &built[2];
        assert!(!setup.2.contains(&"-Werror".to_string()));
        assert!(
            setup
                .2
                .contains(&"/sdk/C_API/buildsupport/setup.c".to_string())
        );
        assert!(setup.2.contains(&"-mword-relocations".to_string()));
    }

    #[test]
    fn the_device_link_uses_the_sdks_linker_script_and_keeps_relocations() {
        let steps = plan(Path::new("/repo"), BOTH, &tools(), "0.2.0").unwrap();
        let link = &runs(&steps)[3].2;
        assert_eq!(
            link[..3],
            [
                "/repo/target/playdate/obj/shim.o",
                "/repo/target/playdate/obj/setup.o",
                "/repo/target/thumbv7em-none-eabihf/playdate/libtrpg_playdate.a",
            ]
        );
        for flag in [
            "-nostartfiles",
            "-mfpu=fpv5-sp-d16",
            "-T/sdk/C_API/buildsupport/link_map.ld",
            "-Wl,-Map=/repo/target/playdate/obj/pdex.map,--cref,--gc-sections,--no-warn-mismatch,--emit-relocs",
        ] {
            assert!(link.contains(&flag.to_string()), "{flag}: {link:?}");
        }
        assert_eq!(
            link[link.len() - 2..],
            ["-o", "/repo/target/playdate/Source/pdex.elf"]
        );
    }

    #[test]
    fn the_simulator_links_a_dll_with_mingw() {
        let steps = plan(Path::new("/repo"), BOTH, &tools(), "0.2.0").unwrap();
        let (_, program, args, _) = &runs(&steps)[5];
        assert_eq!(program.replace('\\', "/"), "/mingw/gcc");
        for flag in [
            "-shared",
            "-s",
            "-D_WINDLL",
            "-DTARGET_SIMULATOR=1",
            "-DTARGET_EXTENSION=1",
            "-static-libgcc",
        ] {
            assert!(args.contains(&flag.to_string()), "{flag}: {args:?}");
        }
        for file in [
            "/repo/crates/playdate/shim/shim.c",
            "/sdk/C_API/buildsupport/setup.c",
            "/repo/target/x86_64-pc-windows-gnu/playdate/libtrpg_playdate.a",
        ] {
            assert!(args.contains(&file.to_string()), "{file}: {args:?}");
        }
        assert_eq!(
            args[args.len() - 2..],
            ["-o", "/repo/target/playdate/Source/pdex.dll"]
        );
    }

    #[test]
    fn pdc_packs_the_source_folder_into_the_pdx() {
        let steps = plan(Path::new("/repo"), BOTH, &tools(), "0.2.0").unwrap();
        let (_, program, args, _) = runs(&steps).pop().unwrap();
        assert_eq!(
            program.replace('\\', "/"),
            format!("/sdk/bin/pdc{}", env::consts::EXE_SUFFIX)
        );
        assert_eq!(
            args,
            [
                "-sdkpath",
                "/sdk",
                "/repo/target/playdate/Source",
                "/repo/target/playdate/VisionsOfShuyi.pdx"
            ]
        );
    }

    #[test]
    fn one_target_builds_only_its_half() {
        let device = Options {
            simulator: false,
            ..BOTH
        };
        let whats: Vec<&str> = runs(&plan(Path::new("/r"), device, &tools(), "0.2.0").unwrap())
            .iter()
            .map(|r| r.0)
            .collect();
        assert!(whats.iter().all(|w| !w.contains("Simulator")), "{whats:?}");
        assert_eq!(whats.len(), 5);
        let simulator = Options {
            device: false,
            ..BOTH
        };
        let whats: Vec<&str> = runs(&plan(Path::new("/r"), simulator, &tools(), "0.2.0").unwrap())
            .iter()
            .map(|r| r.0)
            .collect();
        assert_eq!(
            whats,
            [
                "cargo build (Simulator)",
                "link pdex.dll (Simulator)",
                "pdc"
            ]
        );
    }

    #[test]
    fn a_device_build_without_the_device_tools_says_what_to_install() {
        let device = Options {
            simulator: false,
            ..BOTH
        };
        let no_gcc = Tools {
            arm_gcc: None,
            ..tools()
        };
        let error = plan(Path::new("/r"), device, &no_gcc, "0.2.0").unwrap_err();
        assert!(
            error.contains("arm-none-eabi-toolchain") && error.contains("gcc-arm-none-eabi"),
            "{error}"
        );
        let no_target = Tools {
            targets: (false, true),
            ..tools()
        };
        let error = plan(Path::new("/r"), device, &no_target, "0.2.0").unwrap_err();
        assert!(
            error.contains("rustup target add thumbv7em-none-eabihf"),
            "{error}"
        );
        // The Simulator's tools are not needed for the device.
        let device_only = Tools {
            gcc: None,
            targets: (true, false),
            windows: false,
            ..tools()
        };
        assert!(plan(Path::new("/r"), device, &device_only, "0.2.0").is_ok());
    }

    #[test]
    fn a_simulator_build_without_its_tools_says_what_is_missing() {
        let simulator = Options {
            device: false,
            ..BOTH
        };
        let linux = Tools {
            windows: false,
            ..tools()
        };
        let error = plan(Path::new("/r"), simulator, &linux, "0.2.0").unwrap_err();
        assert!(
            error.contains("Windows only") && error.contains("--device"),
            "{error}"
        );
        let no_gcc = Tools {
            gcc: None,
            ..tools()
        };
        let error = plan(Path::new("/r"), simulator, &no_gcc, "0.2.0").unwrap_err();
        assert!(error.contains("mingw-w64-x86_64-gcc"), "{error}");
        let no_target = Tools {
            targets: (true, false),
            ..tools()
        };
        let error = plan(Path::new("/r"), simulator, &no_target, "0.2.0").unwrap_err();
        assert!(
            error.contains("rustup target add x86_64-pc-windows-gnu"),
            "{error}"
        );
        // The device's tools are not needed for the Simulator.
        let simulator_only = Tools {
            arm_gcc: None,
            targets: (false, true),
            ..tools()
        };
        assert!(plan(Path::new("/r"), simulator, &simulator_only, "0.2.0").is_ok());
    }

    #[test]
    fn executing_makes_fresh_folders_writes_files_and_runs_programs() {
        let dir = scratch("execute");
        let folder = dir.join("Source");
        fs::create_dir_all(&folder).unwrap();
        fs::write(folder.join("stale.dll"), "old").unwrap();
        let file = folder.join("pdxinfo");
        let steps = [
            Step::Fresh(folder.clone()),
            Step::Write {
                path: file.clone(),
                text: "name=x\n".into(),
            },
            Step::Run {
                what: "cargo version",
                program: PathBuf::from("cargo"),
                args: os_args(["--version"]),
                rustflags: Some("-D warnings".into()),
            },
        ];
        assert_eq!(execute(&steps), Ok(()));
        assert!(!folder.join("stale.dll").exists());
        assert_eq!(fs::read_to_string(&file).unwrap(), "name=x\n");
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn executing_stops_at_a_program_that_fails_or_is_missing() {
        let dir = scratch("execute-fails");
        let after = dir.join("after");
        let failing = [
            Step::Run {
                what: "bad cargo",
                program: PathBuf::from("cargo"),
                args: os_args(["--no-such-flag"]),
                rustflags: None,
            },
            Step::Fresh(after.clone()),
        ];
        let error = execute(&failing).unwrap_err();
        assert!(error.starts_with("bad cargo failed"), "{error}");
        assert!(!after.exists());
        let missing = [Step::Run {
            what: "nothing",
            program: dir.join("no-such-program"),
            args: vec![],
            rustflags: None,
        }];
        assert!(execute(&missing).unwrap_err().contains("could not start"));
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn the_summary_names_the_pdx_and_the_sizes_built() {
        let root = scratch("summary");
        let source = out_dir(&root).join("Source");
        fs::create_dir_all(&source).unwrap();
        fs::write(source.join("pdex.elf"), [0; 10]).unwrap();
        let line = summary(&root, BOTH);
        assert!(line.contains("VisionsOfShuyi.pdx"), "{line}");
        assert!(
            line.ends_with("(pdex.elf 10 bytes, pdex.dll missing)"),
            "{line}"
        );
        let device = Options {
            simulator: false,
            ..BOTH
        };
        assert!(summary(&root, device).ends_with("(pdex.elf 10 bytes)"));
        fs::remove_dir_all(&root).unwrap();
    }
}
