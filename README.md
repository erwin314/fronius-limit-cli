# Fronius Limit CLI

A fast, standalone command-line tool written in Rust to quickly set or disable the power export limit on your Fronius solar inverter.

## Usage

The tool allows you to specify a new power limit in Watts, or unset the limit entirely by providing a negative value (e.g. `-1`).

The inverter's base URL and "service" user password are required. You can provide these either via command-line arguments or via environment variables (`FRONIUS_BASE_URL` and `FRONIUS_SERVICE_PASSWORD`).

### Set a limit

To set an export limit of 5000 Watts (5kW):

```bash
fronius-limit-cli --base-url http://192.168.1.100 --password YourPassword! 5000
```

Or, using environment variables for convenience:

```bash
export FRONIUS_BASE_URL="http://192.168.1.100"
export FRONIUS_SERVICE_PASSWORD="YourPassword!"

fronius-limit-cli 5000
```

### Using a `.env` file

Instead of exporting environment variables in your shell, you can create a `.env` file in the working directory. The tool will automatically load it on startup.
You can have a look at [.env.example](.env.example) for an example of what the contents should look like.

### Unset a limit

To disable the export limit entirely and allow maximum feed-in, provide a negative value such as `-1`:

```bash
fronius-limit-cli --base-url http://192.168.1.100 --password YourPassword! -1
```

## Downloads & Distribution Binaries

Pre-compiled binaries are built automatically and available for download in the **Releases** section. Download the appropriate binary for your system:

| Binary Asset Name | Architecture | Target Environment |
| :--- | :--- | :--- |
| `fronius-limit-cli-linux-amd64` | Linux x86_64 | Standard 64-bit Linux environments (e.g., Ubuntu, Debian, standard servers). |
| `fronius-limit-cli-linux-arm64` | Linux ARM64 | Modern ARM environments (e.g., 64-bit Raspberry Pi OS on Pi 3/4/5, AWS Graviton servers). |
| `fronius-limit-cli-linux-armv7` | Linux ARMv7 | Older 32-bit ARM environments (e.g., older Raspberry Pi hardware or running 32-bit OS). |
| `fronius-limit-cli-linux-musl-armv7` | Linux ARMv7 (musl) | Statically-linked 32-bit ARM (specifically intended for **Victron CerboGX** / Venus OS and similar embedded devices). |
| `fronius-limit-cli-linux-musl-amd64` | Linux x86_64 (musl) | Minimal Linux environments requiring fully statically-linked binaries (e.g., Alpine Linux, slim Docker containers). |
| `fronius-limit-cli-macos-amd64` | macOS x86_64 | Macs with Intel processors. |
| `fronius-limit-cli-macos-arm64` | macOS ARM64 | Modern Macs with Apple Silicon processors (M1/M2/M3/M4). |
| `fronius-limit-cli-windows-amd64.exe` | Windows x86_64 | Standard Windows PCs and laptops. |
| `fronius-limit-cli-windows-arm64.exe`| Windows ARM64 | Modern Windows on ARM devices (e.g., new Snapdragon X laptops, Surface Pro X). |

## Releasing a new version
This project is configured to work with `cargo-release` to bump versions, tag, and publish to GitHub. 

To install the tool locally:
```bash
cargo install cargo-release
```
Note: If you get an error about your rustc version being too old run `rustup update`.

To publish a new release:
1. Dry-run the release to make sure everything looks right:
```bash
cargo release minor # (Or patch, major)
```
2. Actually execute the bump, tag, and push:
```bash
cargo release minor --execute
```
*The GitHub Actions workflow will automatically catch the pushed `v*` tag and build the new release binaries.*
