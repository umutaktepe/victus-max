# Contributing to Victus Max

Thank you for your interest in contributing to **Victus Max**! As an open-source tool aiming to provide the best HP Victus and OMEN hardware control on Linux, we welcome all pull requests—from typo fixes to new reverse-engineered board capabilities.

## Development Environment Setup

1. **Prerequisites:**
   - Rust toolchain (via `rustup`, version \(\ge 1.85\))
   - `libgtk-4-dev`, `libadwaita-1-dev`, and `libgtk4-layer-shell-dev`
   - `libsystemd-dev`, `libdbus-1-dev`, `libhidapi-dev`, `pkg-config`
   - Linux kernel headers (for testing the DKMS module)

2. **Local Compilation:**
   Instead of installing system-wide, you can build and test components locally:
   ```bash
   # Build the entire workspace
   cargo build

   # Run tests
   cargo test --workspace

   # Test GUI locally
   cargo run -p victus-max-gui

   # Test CLI locally
   cargo run -p victus-max-cli -- fetch
   ```

## Architectural Rules for Contributors

1. **No Root in Client Applications:**
   - Victus Max strictly follows a split privilege model. `victus-max` (GUI), `victus-max-overlay` (HUD), `victus-max-cli` (CLI), and `victus-max-tray` (Tray) must **never** require `sudo`. 
   - If you need to access a new `/sys/` or `/dev/` endpoint, that logic MUST be written in `victus-max-daemon`.
   - Client applications communicate with the daemon via D-Bus (`zbus`).

2. **Hardware Protection & Safety First:**
   - Always preserve the **10-second asynchronous stagger gap** between fan writes in `fan/mod.rs` to protect HP Victus EC registers.
   - Always maintain the **88°C emergency thermal bypass** in Better Auto.

3. **D-Bus Interface Definitions:**
   - Common types and traits live in `src/victus-max-types/`.
   - Update the XML policies in `data/org.hp.omen.conf` if you introduce new D-Bus methods, ensuring members of the `omen-hw` group can access them.

4. **Asynchronous Code (Tokio):**
   - The GUI thread (GTK) must never be blocked. When calling the D-Bus daemon from client apps, use `glib::spawn_future_local` or `tokio::spawn`.
   - Avoid `std::thread::sleep`—use `tokio::time::sleep`.

## Code Style (Rust)
- Run `cargo fmt` before submitting your PR.
- Run `cargo clippy --workspace -- -D warnings` to ensure zero warnings.
- Keep variable names descriptive. Use `snake_case` for variables/functions and `CamelCase` for structs/enums.

## Submitting a Pull Request
1. Fork the repository (`https://github.com/umutaktepe/victus-max`).
2. Create a feature branch: `git checkout -b feature/my-cool-feature`.
3. Commit your changes with clear, descriptive commit messages.
4. Push to your fork and open a PR against the `main` branch.
5. In your PR description, explain **what** you changed, **why**, and include verification evidence.
