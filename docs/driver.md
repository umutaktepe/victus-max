# Kernel Driver (`hp-omen-extra` DKMS)

The `driver/` directory contains a companion C-based Linux kernel module (`hp-omen-extra`) designed to coexist seamlessly alongside the mainline `hp-wmi` driver.

## Why is a kernel module needed?
For many functions (like fan control, CPU power limits, and reading thermal sensors), Victus Max interacts directly with standard Linux kernel interfaces (like `sysfs`, `hwmon`, and standard WMI-ACPI events). 

However, modern HP Victus and OMEN laptops utilize proprietary, vendor-specific WMI and USB endpoints for **Keyboard RGB Lighting** (especially 4-zone and per-key RGB models). The mainline Linux kernel (`hid-hp`) does not expose controls for these advanced lighting matrices.

## Responsibilities
1. **Hardware Communication:** Sends the precise hexadecimal byte payloads required by the keyboard's embedded microcontroller to change colors, zones, and animations.
2. **Exposing Clean Sysfs Nodes:** The driver mounts nodes in `/sys/devices/platform/hp-omen-extra/` that allow user-space daemons with proper group permissions to read and write lighting data safely.
3. **DKMS Integration:** 
   - Distributed as a DKMS (Dynamic Kernel Module Support) package.
   - Whenever you update your Linux kernel (e.g., from `6.8` to `6.11`), the driver automatically recompiles itself for the new kernel version during boot.
4. **Non-Clashing WMI GUID Architecture:**
   - Unlike legacy monolithic drivers, `hp-omen-extra` does not claim ownership of primary WMI GUIDs (`5FB7D087-05BE-445c-9CE6-1F64EEA549E7`), ensuring standard laptop hotkeys, flight mode switches, and battery sensors never break.

## How it interacts with Victus Max
`victus-max-daemon` contains a `lighting.rs` module. When the user selects a color or lighting effect in `victus-max`, the daemon translates this color into a raw byte buffer and pipes it directly into the kernel driver's exposed endpoint, triggering an instant hardware-level color change on the keyboard.
