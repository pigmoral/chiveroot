use std::process::Command;

mod builder;
mod error;
mod target;

use builder::InitramfsBuilder;
use clap::Parser;
use error::Result;

#[derive(Parser, Debug)]
#[command(name = "chiveroot")]
#[command(version = "0.1.0")]
#[command(
    about = "Build chivebox initramfs for embedded systems",
    long_about = "A tool to build initramfs containing chivebox (a BusyBox-style multi-call binary) with optional kernel modules and firmware."
)]
struct Args {
    #[arg(
        short,
        long,
        help = "Target architecture (e.g., riscv64, arm64, x86_64)"
    )]
    target: Option<String>,

    #[arg(short, long, help = "Output directory or file (default: /tmp)")]
    output: Option<String>,

    #[arg(short, long, help = "Kernel modules to include (file or directory)")]
    modules: Option<String>,

    #[arg(
        short,
        long,
        help = "Kernel version for modules directory (e.g., 5.15.0)"
    )]
    kernel_version: Option<String>,

    #[arg(long, help = "Firmware files to include (file or directory)")]
    firmware: Option<String>,

    #[arg(
        short,
        long,
        help = "Additional files to include (format: src:dst, can be repeated)"
    )]
    file: Vec<String>,

    #[arg(
        short,
        long,
        help = "Path to pre-built chivebox binary (default: build from source)"
    )]
    binary: Option<String>,

    #[arg(
        long,
        help = "Path to chivebox source directory (use with --binary if not providing pre-built binary)"
    )]
    source: Option<String>,

    #[arg(long, help = "List supported target architectures")]
    list_targets: bool,
}

fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let args = Args::parse();

    if args.list_targets {
        println!("Supported targets:");
        for (short, full) in target::SUPPORTED_TARGETS {
            println!("  {:12} -> {}", short, full);
        }
        return Ok(());
    }

    let target = args
        .target
        .ok_or_else(|| error::Error::UnknownTarget("--target is required".to_string()))?;

    let target_triple = target::resolve_target(&target)?;

    // Check that either --source or --binary is provided
    if args.source.is_none() && args.binary.is_none() {
        return Err(error::Error::MissingToolchain(
            "Either --source or --binary must be provided".to_string(),
        ));
    }

    println!("Building initramfs for target: {}", target_triple);

    check_cargo_zigbuild()?;

    let binary_path = if let Some(binary) = args.binary {
        println!("Using provided binary: {}", binary);
        std::path::PathBuf::from(binary)
    } else {
        let source_dir = args
            .source
            .map(std::path::PathBuf::from)
            .expect("--source is required when --binary is not provided");
        build_chivebox(&source_dir, &target_triple.as_str())?
    };

    let applets = get_applet_list(&binary_path)?;

    let output_path = args.output.unwrap_or_else(|| "/tmp".to_string());

    let short_target = target::get_short_name(&target_triple);

    let builder = InitramfsBuilder::new(
        output_path,
        short_target,
        applets,
        args.modules,
        args.kernel_version,
        args.firmware,
        args.file,
    )?;

    let result = builder.build(&binary_path)?;
    println!("Initramfs created: {}", result.display());

    Ok(())
}

fn check_cargo_zigbuild() -> Result<()> {
    let output = Command::new("cargo-zigbuild").arg("--version").output();

    match output {
        Ok(o) if o.status.success() => {
            println!("cargo-zigbuild found");
            Ok(())
        }
        _ => Err(error::Error::MissingToolchain(
            "cargo-zigbuild not found. Please install it:\n\
                 cargo install cargo-zigbuild"
                .to_string(),
        )),
    }
}

fn build_chivebox(source_dir: &std::path::Path, target: &str) -> Result<std::path::PathBuf> {
    println!("Building chivebox from source...");

    let binary_dir = source_dir.join("target").join(target).join("release");

    if binary_dir.join("chivebox").exists() {
        println!("Using existing build at {:?}", binary_dir.join("chivebox"));
        return Ok(binary_dir.join("chivebox"));
    }

    println!("Building chivebox (this may take a while)...");

    let status = Command::new("cargo")
        .args(["zigbuild", "--release", "--target", target])
        .current_dir(source_dir)
        .status()?;

    if !status.success() {
        return Err(error::Error::BuildFailure(
            "chivebox build failed".to_string(),
        ));
    }

    Ok(binary_dir.join("chivebox"))
}

fn get_applet_list(binary_path: &std::path::Path) -> Result<Vec<String>> {
    println!("Getting applet list from chivebox...");

    let output = Command::new(binary_path).arg("--list").output()?;

    if !output.status.success() {
        return Err(error::Error::AppletListFailed(
            "chivebox --list failed".to_string(),
        ));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let applets: Vec<String> = stdout
        .lines()
        .filter_map(|line| {
            let name = line.split_whitespace().next()?;
            if name.starts_with("--") || name.is_empty() {
                None
            } else {
                Some(name.to_string())
            }
        })
        .collect();

    println!("Found {} applets: {:?}", applets.len(), applets);
    Ok(applets)
}
