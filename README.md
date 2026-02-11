# Paqet CLI Automator

Automated network discovery and configuration generator for Paqet.

## How to use

Select the command for your Operating System to download and run the automator:

### Linux (VPS)
```bash
curl -L -O https://github.com/free-mba/paqet-cli-client/releases/download/v0.1.1/paqet-cli-x86_64-unknown-linux-musl
chmod +x paqet-cli-x86_64-unknown-linux-musl
sudo ./paqet-cli-x86_64-unknown-linux-musl
```

### macOS (Apple Silicon - M1/M2/M3)
```bash
curl -L -O https://github.com/free-mba/paqet-cli-client/releases/download/v0.1.1/paqet-cli-aarch64-apple-darwin
chmod +x paqet-cli-aarch64-apple-darwin
sudo ./paqet-cli-aarch64-apple-darwin
```

### macOS (Intel)
```bash
curl -L -O https://github.com/free-mba/paqet-cli-client/releases/download/v0.1.1/paqet-cli-x86_64-apple-darwin
chmod +x paqet-cli-x86_64-apple-darwin
sudo ./paqet-cli-x86_64-apple-darwin
```

### Windows
1. Download `paqet-cli-x86_64-pc-windows-msvc.exe` from the [Releases](https://github.com/free-mba/paqet-cli-client/releases/) page.
2. Run it as Administrator.

This will automatically create an `auto_client.yaml` file with the correct network settings for your machine.
