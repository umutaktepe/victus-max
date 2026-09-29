# Victus Max Tray (`victus-max-tray`)

The `victus-max-tray` crate provides a lightweight (<2MB memory footprint), unobtrusive system tray applet (StatusNotifierItem / AppIndicator) for Victus Max. It runs continuously in the background to provide quick hardware toggles without eating up system resources.

## Responsibilities & Features

1. **Background Presence:**
   - Runs independently of the main GUI application. 
   - Uses the `ksni` crate to register itself seamlessly in the system tray of modern desktop environments like GNOME (via AppIndicator extension), KDE Plasma, XFCE, and wlroots-based compositors (Sway, Hyprland).
2. **Quick Actions Context Menu:**
   - **Open Victus Max:** Spawns or focuses the main `victus-max` control center.
   - **Open Overlay (Shift+F2):** Triggers the floating gaming HUD overlay (`victus-max-overlay`).
   - **Fan Modes Submenu:** Real-time checkmark selector for:
     - `Better Auto` (Proactive load-aware cooling)
     - `Auto` (OEM HP BIOS curve)
     - `Max` (100% Turbo Boost)
     - `Hardware (EC)` (Firmware pass-through)
   - **Power Profiles Submenu:** Instant switching between `Performance`, `Balanced`, and `Quiet`.
   - **Exit:** Gracefully terminates the tray applet and associated user-space interface processes.
3. **D-Bus Integration:**
   - Just like `victus-max` and `victus-max-cli`, the tray applet is entirely unprivileged.
   - It asynchronously sends `zbus` messages to the root `victus-max-daemon` over the D-Bus (`org.hp.omen.*` interfaces).
4. **Configuration & Localization:**
   - Reads language preferences from `~/.config/victus-max/gui_config.json` (with backward compatibility for `~/.config/omenspace/`).

## Technical Architecture

### Core Libraries
- **`ksni`**: A Rust library that implements the StatusNotifierItem (SNI) protocol—the modern Linux system tray standard.
- **`zbus`**: High-performance, memory-safe D-Bus client library.
- **`tokio`**: Asynchronous runtime used to dispatch non-blocking D-Bus hardware requests without freezing the tray event loop.

### Code Breakdown (`src/main.rs`)

1. **`struct VictusMaxTray`**: 
   The core structure implementing the `ksni::Tray` trait. Defines the tray icon (`victus-max`) and tooltip.
2. **`fn menu(&self)`**:
   Constructs the drop-down menu hierarchy with `StandardItem`, `RadioItem`, and `SubMenu`.
   Spawns detached `tokio::spawn` tasks upon activation to keep menu interactions snappy.
3. **`get_conn()`**:
   Connects to the D-Bus System Bus.
4. **`set_power_profile()` & `set_fan_mode()`**:
   Executes `conn.call_method()` targeting `/org/hp/omen/Power` and `/org/hp/omen/Fan` endpoints.
