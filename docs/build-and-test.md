# Development: Build and Test

This guide describes how to build Kubeowler from source, run tests, and perform multi-architecture or cross builds. For installation of pre-built binaries, see [installation.md](installation.md).

---

## Prerequisites

- **Rust**: 1.70 or later (stable). Install via [rustup](https://rustup.rs/) or your distribution package manager.
- **Kubernetes**: 1.23+ cluster and kubeconfig with read access (for running the binary against a cluster).

---

## Rust toolchain

### Install Rust (rustup)

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
source ~/.cargo/env
rustc --version
cargo --version
```

### Optional: Homebrew (macOS)

```bash
brew install rust
source ~/.cargo/env
```

---

## Clone and build

```bash
git clone https://github.com/Ghostwritten/kubeowler.git
cd kubeowler
cargo build --release
```

The binary is produced at `target/release/kubeowler`. For a debug build (faster compile, slower run), use `cargo build` (no `--release`).

### Build script

The project provides `build.sh` for convenience:

```bash
chmod +x build.sh
./build.sh --release          # release build
./build.sh --release --test   # release build and run tests
```

---

## Running tests

```bash
cargo test
```

Run tests with output from successful tests:

```bash
cargo test -- --nocapture
```

---

## Code quality

- **Format:** `cargo fmt` (format code); CI checks with `cargo fmt --check`.
- **Lint:** `cargo clippy` (recommended before submitting changes).

---

## Running the binary locally

After `cargo build --release`, run from the project root:

```bash
./target/release/kubeowler check --help
./target/release/kubeowler check -o report.md
```

Set `KUBECONFIG` or use `--config-file` if your kubeconfig is not in the default location.

---

## Multi-architecture and cross-compilation

To build for another architecture (e.g. from x86_64 to arm64):

- **Linux (gnu):** Install the target and a suitable linker (e.g. `gcc-aarch64-linux-gnu` on Debian/Ubuntu), then:

  ```bash
  rustup target add aarch64-unknown-linux-gnu
  cargo build --release --target aarch64-unknown-linux-gnu
  ```

- **Linux (musl, static):** Use [cross](https://github.com/cross-rs/cross) so the resulting binary does not depend on glibc:

  ```bash
  cargo install cross
  cross build --release --target x86_64-unknown-linux-musl
  cross build --release --target aarch64-unknown-linux-musl
  ```

The project’s release workflow uses musl targets and cross for Linux. See [Rust cross-compilation](https://rust-lang.github.io/rustup/cross-compilation.html) and [development-environment.md](development-environment.md) for more detail.

---

## Troubleshooting

- **Build errors:** Try `cargo clean` then `cargo build --release`.
- **Linker errors (cross-compile):** Ensure the target toolchain and linker are installed (e.g. `aarch64-linux-gnu-gcc` for aarch64-gnu).
- **Cluster connection:** See [troubleshooting.md](troubleshooting.md) for runtime and connectivity issues.
