# Building Minecraft-1.12.2-Rust

The canonical upstream repository is `zxy168zxy168-max/Minecraft-1.12.2-Rust`.

The project builds the Rust client and compiles Vulkan shaders during the Cargo build. A source checkout therefore needs the Rust toolchain plus the native graphics/toolchain dependencies used by the selected platform.

## Windows 10/11 x64

Recommended prerequisites:

- current stable Rust toolchain;
- Visual Studio 2022 Build Tools with the Desktop development with C++ workload;
- Windows 10/11 SDK;
- CMake;
- Vulkan SDK / Vulkan loader and headers;
- Python 3.9 or newer for the resource-import tooling.

Build:

```powershell
cargo build --release --bin mc112-client
```

The executable is written to `target/release/mc112-client.exe`.

## Linux

The GitHub Actions build installs a C/C++ toolchain, CMake/Ninja, pkg-config, Vulkan development files, Mesa OpenGL/EGL development files, and the X11/XCB/XKB/Wayland development headers required by the winit/glutin stack.

On Ubuntu-compatible systems the CI package set is:

```text
build-essential cmake ninja-build pkg-config
libvulkan-dev libgl1-mesa-dev libegl1-mesa-dev
libx11-dev libx11-xcb-dev libxcb1-dev
libxkbcommon-dev libxkbcommon-x11-dev
libwayland-dev
```

Build:

```bash
cargo build --release --bin mc112-client
```

The executable is written to `target/release/mc112-client`.

## CI policy

`.github/workflows/ci.yml` is the source-quality gate (`cargo fmt --check` and release `cargo check`). `.github/workflows/build.yml` performs complete release builds on Windows and Linux and publishes the resulting binaries as workflow artifacts.

Runtime tests that need imported Mojang assets, a real display, or a GPU are intentionally separate from these build-only jobs.
