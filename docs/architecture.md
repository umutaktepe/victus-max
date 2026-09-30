# Victus Max Architecture Map & Documentation

This document serves as the technical map for the **Victus Max** project. It outlines how the software interacts with the hardware, the inter-process communication mechanisms, and the layout of the source code.

## 1. High-Level Architecture

Victus Max follows a **Client-Server Architecture** operating locally on the user's Linux machine via **D-Bus**.

```mermaid
graph TD
    subgraph "Hardware & Kernel"
        K[Linux Kernel]
        ACPI[HP WMI / ACPI]
        MSR[MSR Registers / AMD SMU]
        DKMS[hp-omen-extra DKMS]
    end

    subgraph "Root Context (Server)"
        Daemon[victus-max-daemon]
    end

    subgraph "User Context (Clients)"
        GUI[victus-max (GTK4 / Libadwaita)]
        Overlay[victus-max-overlay (HUD)]
        CLI[victus-max-cli]
        Tray[victus-max-tray]
    end
    
    Daemon <-->|Sysfs / Ioctl| K
    Daemon <-->|WMI Calls| ACPI
    Daemon <-->|CPU Control| MSR
    Daemon <-->|RGB Data| DKMS
    
    GUI <-->|D-Bus (org.hp.omen.*)| Daemon
    Overlay <-->|D-Bus (org.hp.omen.*)| Daemon
    CLI <-->|D-Bus (org.hp.omen.*)| Daemon
    Tray <-->|D-Bus (org.hp.omen.*)| Daemon
```

### Why this architecture?
Direct hardware manipulation (changing fan curves, editing CPU MSR registers, modifying WMI endpoints) requires `root` access. 
By placing all hardware logic inside `victus-max-daemon` (which runs as a root systemd service) and having it expose a safe D-Bus API, client applications like `victus-max` and `victus-max-overlay` can run completely unprivileged. This aligns with modern Linux security standards (similar to how NetworkManager or systemd-logind works).

---

## 2. Component Map

The repository is organized into specific directories representing the components:

### 2.1. `src/victus-max-daemon/`
The core backend service.
- **Language:** Rust (tokio asynchronous runtime)
- **Role:** Handles all logic. Reads sensors, modifies power limits (RAPL/RyzenAdj/NVML), sets fan speeds (Better Auto, Auto, Max, Custom, EC), and pushes RGB data.
- **Key Files:**
  - `main.rs`: Entry point, initializes the D-Bus server via `zbus`.
  - `power.rs`: Handles CPU (PL1/PL2, Undervolt via MSR / RyzenAdj) and GPU (NVIDIA TGP / Dynamic Boost PPAB limits).
  - `fan/mod.rs`: Interfaces with HP's EC (Embedded Controller) and WMI to set fan speeds with 10s EC stagger protection and 90s watchdog.
  - `fan/better_auto.rs`: 8-level proactive load-aware cooling engine with hysteresis and acoustic ceiling capping.
  - `lighting.rs`: Pushes raw byte payloads to the kernel driver for RGB zones/keys.
  - `sysmon/`: High-resolution hardware telemetry and PM-safe GPU polling.

### 2.2. `src/victus-max-gui/`
The primary user interface (`victus-max`).
- **Language:** Rust
- **Framework:** GTK4 + LibAdwaita
- **Role:** Presents a beautiful, reactive desktop interface for the user to configure their hardware.
- **Key Files:**
  - `main.rs`: Window initialization, CSS loading, layout structure.
  - `daemon_client.rs`: Contains the `zbus` proxy macros that generate safe Rust methods to talk to the D-Bus API.
  - `performance_control.rs`: UI for thermal profiles (Eco/Balanced/Performance) and Fan Modes (Better Auto/Auto/Max/Custom).
  - `settings.rs`: Acoustic ceiling and minimum fan RPM controls.
  - `keyboardrgb/`: UI for selecting colors and effects.
  - `updater.rs`: Implements GitHub API checking for OTA software updates and `fwupdmgr` for BIOS updates.
  - `asset_resolver.rs`: Ensures `.svg` and `.png` images are loaded from the correct system paths (`/usr/share/victus-max/assets/`).

### 2.3. `src/victus-max-overlay/`
The floating gaming HUD overlay (`victus-max-overlay`).
- **Language:** Rust
- **Framework:** GTK4 + gtk4-layer-shell
- **Role:** Lightweight Wayland/X11 HUD toggled via `Shift + F2` to quickly adjust fan and power modes in-game without alt-tabbing.

### 2.4. `src/victus-max-cli/`
The command-line tool (`victus-max-cli`).
- **Language:** Rust
- **Role:** Allows scripts or power users to control hardware directly from the terminal (e.g., `victus-max-cli fan set-mode better-auto`).

### 2.5. `src/victus-max-tray/`
The system tray icon (`victus-max-tray`).
- **Language:** Rust
- **Role:** Runs quietly in the background, providing quick toggles (right-click menu) for Better Auto, Auto, Max, Power profiles, and launching the overlay or GUI.

### 2.6. `src/victus-max-types/`
Shared D-Bus traits and types.
- **Language:** Rust
- **Role:** Provides strongly typed D-Bus interfaces shared across daemon, GUI, CLI, tray, and overlay.

### 2.7. `driver/`
The kernel module (`hp-omen-extra` DKMS).
- **Language:** C
- **Role:** Provides character device and sysfs interfaces for 4-zone and per-key RGB backlighting where mainline Linux kernel drivers lack vendor support.

### 2.8. `data/`
System integration files.
- `org.hp.omen.conf`: Polkit / D-Bus security policy allowing standard users in the `omen-hw` group to communicate with the root daemon.
- `victus-max-daemon.service`: The systemd root service definition.
- `org.hp.VictusMax.desktop`: The application launcher for Desktop Environments (GNOME, KDE).
- `org.hp.VictusMax.service`: D-Bus activation definition for user sessions.
- `99-victus-max.rules`: Udev rules ensuring sysfs and MSR nodes have correct group permissions.

---

## 3. Data Flow Example: Better Auto Fan Execution

To understand how the app works, here is the lifecycle of cooling automation:

1. **User Action / Background Monitor:** User enables `Better Auto` in `victus-max` or via `Shift + F2` overlay.
2. **GUI Layer:** `daemon_client.rs` calls `SetFanMode("better_auto")`.
3. **D-Bus Layer:** `zbus` delivers the method call to `org.hp.omen.Fan` on the System Bus.
4. **Daemon Telemetry:** Every polling cycle, `victus-max-daemon` calculates CPU load delta via `/proc/stat` and queries active GPU metrics without waking sleeping dGPUs.
5. **Engine Evaluation:** `BetterAutoEngine` calculates the target level (1..=8), evaluates upward/downward hysteresis, clamps to the acoustic ceiling, or triggers emergency bypass if \(\ge 88^\circ\text{C}\).
6. **Hardware Safety:** The daemon writes Fan 1 target RPM, enforces a 10-second asynchronous gap to protect the EC bus, and writes Fan 2 target RPM.
7. **Watchdog:** Every 80–90 seconds, the daemon re-asserts mode control so HP BIOS does not stealthily reclaim the fans.

## 4. Build & Install System
The `setup.sh` script automates compilation using `cargo build --release` for workspace crates, installs binaries to `/usr/bin/` and `/usr/libexec/victus-max/`, provides backward-compatible symlinks (`omen-*`), installs the DKMS module, and restarts systemd services.
