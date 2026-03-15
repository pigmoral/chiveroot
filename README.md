# ChiveRoot

A tool to build chivebox initramfs for embedded systems.

## Features

- Build initramfs with chivebox (BusyBox-style multi-call binary)
- Support multiple target architectures: riscv64, arm64, x86_64, etc.
- Include kernel modules and firmware files
- Automatic applet symlink generation from chivebox `--list`
- Output as cpio.gz archive

## Installation

Build from source:

```bash
git clone https://github.com/pigmoral/chivebox
cd chiveroot
cargo build --release
```

The binary will be at `target/release/chiveroot`.

## Usage

```bash
# List supported targets
chiveroot --list-targets

# Build with pre-built binary
chiveroot --target riscv64 --binary /path/to/chivebox

# Build from source
chiveroot --target riscv64 --source /path/to/chivebox

# Specify output directory
mkdir -p output
chiveroot --target riscv64 --binary /path/to/chivebox --output ./output

# Include kernel modules
chiveroot --target riscv64 --binary /path/to/chivebox --modules /path/to/modules

# Specify kernel version (creates lib/modules/<version>/)
chiveroot --target riscv64 --binary /path/to/chivebox --modules /path/to/modules --kernel-version 5.15.0

# Include firmware files
chiveroot --target riscv64 --binary /path/to/chivebox --firmware /path/to/firmware
```

## Requirements

- `cargo-zigbuild` for cross-compilation

Install with:

```bash
cargo install cargo-zigbuild
```

## Related

ChiveRoot is inspired by [BusyBox](https://busybox.net/) and [u-root](https://github.com/u-root/u-root).

- **BusyBox** is the original Swiss Army knife of embedded Linux, providing many common Unix utilities in a single binary.
- **u-root** is a modern, Go-based alternative that provides a minimalistic initramfs environment with many standard Linux tools.
