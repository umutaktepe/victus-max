# Comprehensive Code Review Guide for Victus Max

This guide is intended for maintainers, senior contributors, and reviewers evaluating Pull Requests (PRs) submitted to the **Victus Max** project. Our primary objective is to maintain a rock-solid, secure, and memory-safe architecture while interacting with sensitive laptop hardware.

---

## 1. Security & Privilege Boundaries (CRITICAL)

Victus Max uses a split-privilege architecture. The user-facing apps (`victus-max`, `victus-max-overlay`, `victus-max-cli`, `victus-max-tray`) run entirely unprivileged, while `victus-max-daemon` runs as root to interact with the kernel and hardware.

- **Zero Sudo in User Space:** 
  If a PR modifies `victus-max`, `victus-max-overlay`, `victus-max-tray`, or `victus-max-cli` to execute shell commands with `sudo` (e.g., `pkexec` or `sudo systemctl`), **reject it immediately**. All hardware logic must be routed through D-Bus to the daemon.
- **Strict D-Bus Input Validation:**
  When reviewing `victus-max-daemon`, ensure that incoming arguments from D-Bus clients are rigorously sanitized.
  - **No Shell Injection:** Never pass raw D-Bus strings into `tokio::process::Command` without strict escaping or parsing.
  - **Type & Bounds Checking:** Ensure strings representing profiles (e.g., `"eco"`, `"balanced"`, `"performance"`) are mapped to internal Enums using `match` statements with a fallback error case. RPM and wattage limits must be clamped to verified ranges.
- **Polkit & D-Bus Policies:**
  If a PR adds a new D-Bus method to `src/victus-max-daemon`, ensure the policy file `data/org.hp.omen.conf` is updated correctly. Access must remain restricted to the `omen-hw` or administrative groups.

---

## 2. Asynchronous Execution & Concurrency

Victus Max relies on the `tokio` asynchronous runtime to keep both the daemon and the GUI/HUD highly responsive.

- **GTK Event Loop Freezing:** 
  In the GUI (`victus-max`) and Overlay (`victus-max-overlay`), heavy D-Bus calls or I/O operations **must not block the main GTK thread**. Look for `glib::spawn_future_local` or `tokio::spawn`. If you spot `std::thread::sleep`, synchronous `std::fs::read_to_string` for large files, or blocking HTTP requests inside UI signal handlers, request the author to switch to asynchronous equivalents.
- **Memory Leaks in GTK Signals:**
  When closures are attached to GTK buttons (e.g., `connect_clicked`), ensure the author correctly uses the `glib::clone!(@weak self)` macro. Capturing strong references inside signal closures will create reference cycles and leak memory.
- **Daemon Concurrency:** 
  Hardware access (like reading slow I2C buses, writing to sysfs, or querying GPU telemetry) can block threads. Ensure these operations are spawned on dedicated blocking threads (`tokio::task::spawn_blocking`) or handled asynchronously without blocking the primary D-Bus event loop.

---

## 3. Hardware Safety (ACPI, EC, MSR, SMU)

HP hardware is highly proprietary and sensitive. Incorrect writes can freeze the Embedded Controller, trigger thermal shutdowns, or cause kernel panics.

- **Embedded Controller (EC) Writes:** 
  Review changes to `fan/mod.rs` and `fan/better_auto.rs` meticulously.
  - Ensure the **10-second asynchronous stagger gap** between Fan 1 and Fan 2 writes is preserved to prevent EC bus register collisions on HP Victus motherboards.
  - Verify that the **90-second BIOS watchdog timer** is maintained.
  - Always verify that the **88°C emergency thermal bypass** is intact.
- **Bounds Checking for Power:** 
  If a PR modifies RAPL (PL1/PL2) limits or MSR undervolting, verify that the daemon enforces hard limits (e.g., clamping fan RPMs to 2000–3500 and ceiling to 3–8).
- **DKMS Module Updates (`driver/`):** 
  If the C kernel module is modified:
  - Check for proper memory allocation and `kfree()` to prevent kernel panics.
  - Prevent null pointer dereferences.
  - Ensure the module hooks correctly into Linux subsystems without clashing with the stock `hp-wmi` module.

---

## 4. Rust Idioms and Standards

- **Clippy and Fmt:** 
  All PRs must pass `cargo clippy --workspace -- -D warnings` and `cargo fmt`. Fix CI/CD pipeline failures before requesting human review.
- **Error Handling (No Panics):** 
  Avoid `unwrap()`, `expect()`, or `panic!()` in production code paths. A failure to read a temperature sensor should never crash the daemon.
  - **In the Daemon:** Return a generic `Result` and log errors via `log::error!`.
  - **In the GUI:** Present a user-friendly Libadwaita dialog (e.g., `adw::MessageDialog`) or an in-app toast notification.
- **Unsafe Code:** 
  Any `unsafe {}` blocks must be highly scrutinized. The author must include a `// SAFETY:` comment explaining exactly why the unsafe block is sound.

---

## 5. D-Bus API Design

- **Backward Compatibility:** 
  Avoid breaking existing D-Bus signatures. When a method signature is updated, ensure GUI, CLI, Tray, and Overlay clients are updated in the same PR.
- **Stateless Daemon:** 
  The daemon should read live hardware state directly from kernel/ACPI/sysfs nodes rather than caching stale states internally, as hardware parameters can be altered by BIOS outside the application.

---

## 6. Review Checklist for Maintainers

Before merging a PR, explicitly verify the following:

- [ ] **Security:** Does this break the unprivileged GUI model? Are inputs sanitized?
- [ ] **Performance:** Is GTK blocking avoided? Are slow operations offloaded to async tasks?
- [ ] **Hardware Safety:** Are the 10s EC stagger gap and 88°C emergency bypass preserved?
- [ ] **Memory Safety:** Are GTK signal closures using weak clones? Is `unsafe` justified?
- [ ] **Error Handling:** Are errors caught gracefully without panicking the application?
- [ ] **Formatting & Tests:** Does `cargo check --workspace` and `cargo test --workspace` pass cleanly?
- [ ] **Documentation:** If a new feature or architectural change was added, were `docs/` and `docs/victus-max-wiki/` updated with an append-only entry in `log.md`?
