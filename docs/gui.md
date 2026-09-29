# Victus Max GUI (`victus-max`)

The graphical user interface for Victus Max is designed to be modern, responsive, and visually cohesive with modern Linux desktop environments (like GNOME and KDE Plasma). It is entirely built in **Rust** using **GTK4** and **LibAdwaita**.

## Responsibilities

1. **User Interaction & Dashboard:**
   - Provides an intuitive dashboard for users to monitor hardware statistics (CPU/GPU temperatures, live fan RPMs, battery status).
   - Allows users to easily toggle Fan Modes (Better Auto, Auto, Max, Custom), Performance Profiles (Eco, Balanced, Performance), and Keyboard RGB animations without touching a terminal.
2. **D-Bus Client Implementation:**
   - The GUI has **no root privileges**. It cannot control hardware directly.
   - It uses `zbus` proxy macros (inside `daemon_client.rs`) to asynchronously send commands to `victus-max-daemon`.
3. **Asset & Theme Management:**
   - Automatically loads CSS styling (`style.css`) for custom UI components (such as elevated glassmorphism cards and neon accent chips).
   - Resolves image and SVG paths dynamically (`asset_resolver.rs`) so the application works seamlessly whether it is launched locally via `cargo run` or installed system-wide in `/usr/share/victus-max/assets/`.
4. **Settings & Acoustic Controls:**
   - Provides intuitive spinners and combo boxes to fine-tune Better Auto's Minimum Fan RPM (2000–3500 RPM) and Acoustic Ceiling (Levels 3–8).
5. **OTA Updates:**
   - Features a dual-channel selector allowing users to switch between **Canary** (latest commits on the `main` branch) and **Stable** (official GitHub Releases).
   - Channel preferences are persisted in `~/.config/victus-max/settings.json` across sessions.
   - Built with a zero-404 fallback mechanism that gracefully falls back to the Canary branch if no official GitHub Releases are published yet, notifying the user with an in-app banner.
   - Executes system-wide updates via an isolated Polkit action (`org.hp.victusmax.update`) and `/usr/libexec/victus-max/victus-max-updater`, keeping the GUI fully unprivileged.
   - Provides live stage progress tracking (`[STAGE:...]`) with a percentage bar and collapsible terminal log viewer, plus a one-click restart button upon completion.
   - Integrates `fwupdmgr` to scan for HP BIOS and firmware updates natively.

## Key Files

- `src/main.rs`: Window initialization, application ID (`org.hp.VictusMax`), sidebar navigation, and CSS loading.
- `src/performance_control.rs`: The UI logic for the "Power & Fans" page. Hosts the Better Auto card, thermal profile cards, and dynamic custom curve presets.
- `src/settings.rs`: Acoustic ceiling and minimum fan RPM controls.
- `src/keyboardrgb/`: Color picker and lighting animation effect selector.
- `src/monitoring.rs`: Real-time hardware telemetry gauges and CPU/GPU temperature meters.
- `src/update_checker.rs`: Async GitHub REST API client, SemVer and commit comparison, zero-404 fallback handling, and persistent settings.json channel management.
- `src/updater.rs`: Manages the OTA update center UI, Polkit pkexec updater invocation, live stage parsing, terminal stream display, and HP BIOS scans.
- `src/i18n.rs`: Full bilingual localization (English and Turkish).
