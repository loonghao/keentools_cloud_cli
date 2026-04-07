# Installation

## Quick Install (Linux / macOS)

```bash
curl -fsSL https://raw.githubusercontent.com/loonghao/keentools_cloud_cli/main/install.sh | bash
```

To install a specific version or to a custom directory:

```bash
bash install.sh --version v0.2.0 --install-dir ~/.local/bin
```

## Quick Install (Windows PowerShell)

```powershell
irm https://raw.githubusercontent.com/loonghao/keentools_cloud_cli/main/install.ps1 | iex
```

To install a specific version:

```powershell
.\install.ps1 -Version v0.2.0 -InstallDir C:\tools
```

## Manual Download

Download the correct binary for your platform from the
[Releases page](https://github.com/loonghao/keentools_cloud_cli/releases/latest):

| Platform            | Asset                                      |
| ------------------- | ------------------------------------------ |
| Linux x86_64 (musl) | `keentools-cloud-x86_64-linux-musl.tar.gz` |
| Linux x86_64 (gnu)  | `keentools-cloud-x86_64-linux-gnu.tar.gz`  |
| Linux ARM64         | `keentools-cloud-aarch64-linux-gnu.tar.gz` |
| macOS Intel         | `keentools-cloud-x86_64-apple-darwin.tar.gz` |
| macOS Apple Silicon | `keentools-cloud-aarch64-apple-darwin.tar.gz` |
| Windows x64         | `keentools-cloud-x86_64-pc-windows-msvc.zip` |

Extract and place the binary somewhere on your `PATH`.

## Build from Source

Requires Rust 1.70+:

```bash
git clone https://github.com/loonghao/keentools_cloud_cli.git
cd keentools_cloud_cli
cargo build --release
# Binary: target/release/keentools-cloud
```

## Verify

```bash
keentools-cloud --version
```

## Self-Update

Once installed, keep it current with:

```bash
keentools-cloud self-update
```
