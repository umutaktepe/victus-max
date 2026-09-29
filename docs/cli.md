# Victus Max CLI (`victus-max-cli`)

The `victus-max-cli` crate provides a lightning-fast command-line interface for power users, script writers, terminal enthusiasts, and headless environments. For backward compatibility, `omen-cli` is also provided as a symlink.

## Responsibilities

1. **Terminal Control:**
   - Exposes every hardware feature available in the GUI directly to the terminal.
   - Designed to be extremely fast. It executes commands and exits immediately, making it ideal for custom shell scripts, shortcuts, or tiling window manager keybindings (e.g., binding a keyboard shortcut to `victus-max-cli fan set-mode max`).
2. **Fastfetch-Style System Summary:**
   - Includes a rich ASCII system summary (`victus-max-cli fetch`) displaying CPU/GPU temperatures, live fan RPMs, power profiles, and battery metrics.
3. **D-Bus Communication:**
   - Like all client components in Victus Max, the CLI runs completely unprivileged. 
   - It acts as an asynchronous D-Bus client, forwarding arguments safely to `victus-max-daemon`.

## Common Usage Examples

### Fan & Cooling Control
```bash
# View fan status, live RPMs, minimum RPM, and acoustic ceiling
victus-max-cli fan info

# Set Fan Modes: better-auto, auto, max, custom, ec
victus-max-cli fan set-mode better-auto
victus-max-cli fan set-mode max
victus-max-cli fan set-mode auto

# Configure Better Auto Acoustic Limits
victus-max-cli fan set-min-rpm 2600     # Range: 2000 - 3500 RPM
victus-max-cli fan get-min-rpm
victus-max-cli fan set-ceiling 5        # Range: Level 3 to Level 8
victus-max-cli fan get-ceiling
```

### Power Profiles & Battery
```bash
# Set thermal/power profiles: performance, balanced, quiet
victus-max-cli power set-profile performance
victus-max-cli power set-profile balanced
victus-max-cli power set-profile quiet

# Limit battery charge to 80% for longevity
victus-max-cli system battery-care 80
```

### System Diagnostics & Maintenance
```bash
# Fastfetch-style visual summary of your laptop
victus-max-cli fetch

# Run heatsink fan dust cleaning ritual
victus-max-cli system clean-fans

# Check for HP BIOS and Victus Max application updates
victus-max-cli system check-bios
victus-max-cli system check-update
```
