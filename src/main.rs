fn main() -> anyhow::Result<()> {
    // Show the renderer/network lifecycle without requiring users to know the
    // RUST_LOG environment variable. An explicit RUST_LOG value still wins.
    let environment = env_logger::Env::default().filter_or("RUST_LOG", "info");
    env_logger::Builder::from_env(environment)
        .format_timestamp_millis()
        .init();
    log::info!(
        "Minecraft 1.12.2 Rust dual-renderer client package {}",
        env!("CARGO_PKG_VERSION")
    );
    minecraft_1_12_2_rust_vulkan::net::minecraft::client::main::Main::main(std::env::args_os())
}
