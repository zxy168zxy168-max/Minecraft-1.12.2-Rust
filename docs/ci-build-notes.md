# CI build dependency notes

The release-build workflow deliberately installs both Vulkan and OpenGL/window-system development dependencies on Linux because the crate contains both rendering backends and uses winit/glutin. Installing only `libvulkan-dev` is not sufficient for a clean Ubuntu runner.

The Windows job uses the Vulkan SDK setup action so the shader/Vulkan build path is available during `cargo build --release --bin mc112-client`.

These jobs validate buildability only. Runtime asset/GPU behavior remains covered by separate manual or environment-specific testing.
