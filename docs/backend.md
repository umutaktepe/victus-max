# Victus Max Backend Daemon (`victus-max-daemon`)

The backend daemon is the absolute core of the Victus Max stack. Because controlling laptop hardware (like fan speeds, CPU power limits, and RGB memory registers) requires strict `root` privileges, the daemon is designed to run in the background as a `systemd` service (`victus-max-daemon.service`) and expose safe methods over D-Bus for user-space interfaces to interact with.

## Responsibilities

1. **Hardware Interfacing:**
   - **WMI / ACPI:** Communicates with HP's ACPI endpoints to change thermal policies (`power-saver`, `balanced`, `performance`).
   - **Embedded Controller (EC):** Reads and writes directly to EC memory to control fan speeds with 10-second asynchronous stagger gaps and a 90-second BIOS watchdog timer.
   - **Better Auto Cooling:** Runs the proactive load-aware 8-level cooling engine, reading `/proc/stat` deltas and PM-safe GPU status.
   - **MSR (Model-Specific Registers) & AMD SMU:** Applies Intel CPU undervolting/TCC offset limits via MSR `0x150`, or tunes AMD Ryzen STAPM/PPT limits natively via `ryzenadj`.
   - **Sysfs & RAPL:** Manages Intel Power Limits (PL1/PL2) directly through `/sys/class/powercap`.
   - **NVIDIA TGP & Dynamic Boost (PPAB):** Automatically manages GPU power targets and unlocks full Dynamic Boost in Performance mode.

2. **D-Bus Server Setup (`org.hp.omen.*`)**
   - The daemon uses the `zbus` Rust crate to bind to the Linux System Bus.
   - It exposes multiple objects/interfaces:
     - `org.hp.omen.Fan`: Better Auto, Auto, Max, Custom, and EC mode switching, minimum RPM, acoustic ceiling.
     - `org.hp.omen.Power`: Power profile management, RAPL/Ryzen limits, battery care (%80 limit).
     - `org.hp.omen.SysMon`: High-resolution telemetry and hardware diagnostic reporting.
     - `org.hp.omen.Rgb`: Static colors, animations, and per-key calibration wizard.
     - `org.hp.omen.Mux`: Display panel routing (Hybrid vs. Discrete).
   - A Polkit/D-Bus security configuration (`data/org.hp.omen.conf`) ensures that only users in the `omen-hw` or `wheel` group can send messages to these endpoints, preventing random unprivileged apps from altering hardware states.

## Key Files

- `src/main.rs`: Entry point. Initializes asynchronous tasks and mounts the D-Bus server.
- `fan/mod.rs` & `fan/better_auto.rs`: EC logic for reading RPMs, Better Auto proactive state machine, and PWM curve injection.
- `power.rs`: Manages TDP/TGP, power limits, and undervolting.
- `lighting.rs`: Parses colors and animations, pushing raw byte packets to the kernel driver.
- `sysmon/`: High-resolution hardware telemetry and diagnostic reports.

## Configuration Persistence
All configuration settings are saved in `/etc/victus-max/` (with automatic fallback to `/etc/omenspace/` for existing setups):
- `/etc/victus-max/power.json`: Power profiles and RAPL limits.
- `/etc/victus-max/fan.json`: Fan modes, minimum RPM, acoustic ceiling.
- `/etc/victus-max/app_profiles.json`: Game-specific automation rules.

## Security Considerations
Since this daemon runs as `root`, it rigorously parses incoming D-Bus arguments to ensure no malicious commands are executed (e.g., strictly validating integer bounds for power rather than executing raw shell strings).
