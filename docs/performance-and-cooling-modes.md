# Victus Max: Performance & Cooling Modes Guide

A comprehensive guide to understanding how performance power profiles and fan cooling modes operate in **Victus Max**, how they interact with HP Victus and OMEN hardware, and how to choose the ideal configuration for any workload.

---

## Table of Contents

1. [Architectural Overview](#architectural-overview)
2. [Performance Power Modes](#performance-power-modes)
   - [Quiet / Eco](#1-quiet--eco-power-saver)
   - [Balanced](#2-balanced-balanced)
   - [Performance](#3-performance-performance)
   - [Under the Hood: Hardware Power Controls](#under-the-hood-hardware-power-controls)
3. [Fan Cooling Modes](#fan-cooling-modes)
   - [Better Auto (Proactive Smart Cooling)](#1-better-auto-better_auto--the-flagship-engine)
   - [Auto (OEM BIOS Curve)](#2-auto-auto)
   - [Max Fan Boost (100% Turbo)](#3-max-fan-boost-max)
   - [Custom Curve (Manual Preset)](#4-custom-curve-custom)
   - [Hardware EC Mode (Pure Firmware Fallback)](#5-hardware-ec-mode-ec)
4. [Power & Fan Synchronization Matrix](#power--fan-synchronization-matrix)
5. [Real-World Scenarios & Walkthroughs](#real-world-scenarios--walkthroughs)
   - [Scenario 1: Daily Productivity & Office Work](#scenario-1-daily-productivity--office-work)
   - [Scenario 2: High-End & Competitive Gaming](#scenario-2-high-end--competitive-gaming)
   - [Scenario 3: Library, Classroom, or Late-Night Quiet Study](#scenario-3-library-classroom-or-late-night-quiet-study)
   - [Scenario 4: Heavy Code Compilation & 3D Rendering](#scenario-4-heavy-code-compilation--3d-rendering)
   - [Scenario 5: Periodic Heatsink Dust Cleaning](#scenario-5-periodic-heatsink-dust-cleaning)
6. [Quick Reference Cheat Sheet](#quick-reference-cheat-sheet)
7. [Control Interfaces (GUI, HUD, CLI, Tray)](#control-interfaces)

---

## Architectural Overview

Laptops require careful orchestration between two distinct systems:
1. **Power & Clocks:** How many watts the CPU and GPU are allowed to consume.
2. **Cooling & Thermals:** How fast the intake/exhaust fans spin to dissipate that heat.

In factory OEM setups (such as HP OMEN Gaming Hub or stock Windows software), fan control is purely **reactive**: the system waits until thermal sensors hit high temperatures before ramping up fans. Because laptop copper heatpipes and vapor chambers take time to absorb heat, reactive cooling leads to annoying fan noise surges ("fan hunting") and thermal throttling before the fans even reach target RPM.

**Victus Max** solves this by separating power control from fan control while establishing an intelligent, proactive bridge between them:

```
┌────────────────────────────────────────────────────────────────────────┐
│                        Victus Max User Interfaces                      │
│   GTK4 Control Center  •  Shift+F2 Overlay HUD  •  CLI  •  System Tray │
└───────────────────────────────────┬────────────────────────────────────┘
                                    │ D-Bus IPC
┌───────────────────────────────────▼────────────────────────────────────┐
│                    victus-max-daemon (Root Service)                    │
│                                                                        │
│   ┌──────────────────────────────┐   ┌──────────────────────────────┐  │
│   │     Power Profile Service    │   │      Fan Cooling Service     │  │
│   │   • ACPI platform_profile    │───►   • Better Auto Engine       │  │
│   │   • Intel RAPL / AMD RyzenAdj│   │   • Proactive /proc/stat load│  │
│   │   • NVIDIA TGP & PPAB Boost  │   │   • Acoustic Ceiling Sync    │  │
│   └──────────────┬───────────────┘   └──────────────┬───────────────┘  │
└──────────────────┼──────────────────────────────────┼──────────────────┘
                   │                                  │
┌──────────────────▼──────────────────────────────────▼──────────────────┐
│                           Laptop Hardware                              │
│   • CPU (Intel/AMD)  • GPU (NVIDIA dGPU)  • HP Embedded Controller (EC)│
└────────────────────────────────────────────────────────────────────────┘
```

---

## Performance Power Modes

Victus Max provides three primary performance power profiles. Each profile controls CPU wattage caps, GPU Dynamic Boost targets, and operating voltage offsets.

### 1. Quiet / Eco (`power-saver`)
* **Target Audience:** Library work, battery conservation, web reading, late-night media consumption.
* **CPU Behavior:** Limits CPU power (PL1/PL2 on Intel or STAPM on AMD) to minimal operational levels (e.g., 15W–25W depending on model). Prevents sustained turbo boost frequencies.
* **GPU Behavior:** NVIDIA discrete GPU is kept in power-saving sleep (`runtime_status: suspended` / D3cold) whenever unneeded. Dynamic boost (PPAB) is disabled.
* **Thermal Profile:** Sets ACPI `platform_profile` to `low-power` / `quiet`.
* **Acoustics:** Fans operate at minimum RPM or whisper-quiet levels.

### 2. Balanced (`balanced`)
* **Target Audience:** Standard day-to-day computing, software development, video playback, casual gaming.
* **CPU Behavior:** Standard manufacturer power limits (e.g., PL1: 45W, PL2: 80W). The CPU boosts dynamically for short bursts (opening apps, compiling small files) and settles into efficient clock speeds during sustained work.
* **GPU Behavior:** NVIDIA dGPU dynamically boosts as needed by graphical workloads, returning to sleep when on desktop.
* **Thermal Profile:** Sets ACPI `platform_profile` to `balanced`.
* **Acoustics:** Balanced between cool surface temperatures and low fan noise. Governed by the **Acoustic Ceiling** (default Level 5, ~4100 RPM).

### 3. Performance (`performance`)
* **Target Audience:** Triple-A gaming, esports, Unreal Engine editing, 3D rendering, machine learning inference.
* **CPU Behavior:** Unlocks full sustained power limits (e.g., PL1: 65W–90W+, PL2: 115W–140W+). CPU maintains maximum all-core boost clocks as long as thermals permit.
* **GPU Behavior:** Enables full NVIDIA TGP (Total Graphics Power) and triggers HP Dynamic Boost (`ppab` register set to `1`).
* **Thermal Profile:** Sets ACPI `platform_profile` and WMI thermal policy to `performance` (or thermal profile `1`).
* **Acoustics:** Acoustic ceilings are automatically lifted. Fans are granted full access to Level 8 (100% Turbo speed) to maximize clock stability and prevent thermal throttling.

### Under the Hood: Hardware Power Controls
When you select a power mode, `victus-max-daemon` writes to the following hardware subsystems:
- **ACPI Platform Profile:** `/sys/firmware/acpi/platform_profile` (or HP WMI platform profile nodes).
- **Intel RAPL:** `/sys/class/powercap/intel-rapl/intel-rapl:0/constraint_*_power_limit_uw` (direct milliwatt register writes).
- **AMD Ryzen:** Communicates via `ryzenadj` (STAPM, Slow PPT, and Fast PPT parameters).
- **NVIDIA Dynamic Boost (PPAB):** Activates `/sys/devices/platform/hp-wmi/ppab` and GPU TGP nodes to unlock the maximum GPU wattage rating.
- **Power-to-Fan Notification:** Dispatches an internal D-Bus notification (`NotifyPowerProfile`) to the fan service to instantly align acoustic limits.

---

## Fan Cooling Modes

Victus Max features five fan cooling modes tailored for different hardware capabilities and user preferences.

```
┌────────────────────────────────────────────────────────────────────────┐
│                        Victus Max Fan Modes                            │
├──────────────┬──────────────────┬──────────────────────────────────────┤
│ Mode Name    │ Engine           │ Key Characteristic                   │
├──────────────┼──────────────────┼──────────────────────────────────────┤
│ Better Auto  │ Victus Max Rust  │ Proactive load-aware, anti-hunting   │
│ Auto         │ OEM HP BIOS      │ Reactive OEM thermal table curve     │
│ Max Boost    │ 100% Turbo Lock  │ Maximum continuous cooling (~5000+RPM)│
│ Custom       │ User Spline/Step │ User-configured manual RPM points    │
│ Hardware EC  │ Hardware EC      │ Safe firmware fallback (no software) │
└──────────────┴──────────────────┴──────────────────────────────────────┘
```

---

### 1. Better Auto (`better_auto`) — The Flagship Engine

**Better Auto** is a next-generation cooling algorithm originally inspired by `victus-control` and completely re-engineered in pure Rust for Victus Max.

#### The Problem It Solves
Standard laptop fans only measure temperatures. However:
- Temperatures lag behind CPU/GPU compute spikes by 3 to 10 seconds.
- By the time heat reaches the thermal diode, the copper heatsink is already saturated.
- Fans suddenly scream at high RPM, then slow down, then scream again ("fan hunting").

#### How Better Auto Works
Better Auto continuously monitors two metrics simultaneously every polling cycle:
1. **Live Temperature (°C):** Highest reading among CPU and GPU thermal packages.
2. **Instantaneous Workload (%):** Precise delta calculations from `/proc/stat` for CPU usage, combined with power-management-safe GPU utilization queries (runtime status is checked first so sleeping NVIDIA GPUs are never woken up on battery).

#### The 8-Step Proactive Matrix
Better Auto maps temperatures and workload usages to an 8-level scale:

| Level | Upward Temp (°C) | Downward Temp (°C) | Workload Usage (%) | Target Fan Speed | Approx RPM Range |
|:-----:|:----------------:|:------------------:|:------------------:|:----------------:|:----------------:|
| **1** | Baseline (<45°C) | Baseline (<42°C)   | Baseline (<25%)    | Minimum RPM      | 2000 – 2600 RPM  |
| **2** | \(\ge 45^\circ\text{C}\) | \(\le 42.0^\circ\text{C}\) | \(\ge 25\%\) | Level 2 Step | ~2900 RPM |
| **3** | \(\ge 54^\circ\text{C}\) | \(\le 51.0^\circ\text{C}\) | \(\ge 35\%\) | Level 3 Step | ~3300 RPM |
| **4** | \(\ge 62^\circ\text{C}\) | \(\le 59.0^\circ\text{C}\) | \(\ge 48\%\) | Level 4 Step | ~3700 RPM |
| **5** | \(\ge 68^\circ\text{C}\) | \(\le 65.0^\circ\text{C}\) | \(\ge 58\%\) | Level 5 Step | ~4100 RPM |
| **6** | \(\ge 73^\circ\text{C}\) | \(\le 69.5^\circ\text{C}\) | \(\ge 66\%\) | Level 6 Step | ~4500 RPM |
| **7** | \(\ge 78^\circ\text{C}\) | \(\le 74.5^\circ\text{C}\) | \(\ge 74\%\) | Level 7 Step | ~4900 RPM |
| **8** | \(\ge 83^\circ\text{C}\) | \(\le 79.0^\circ\text{C}\) | \(\ge 82\%\) | Max Turbo (100%) | 5200 – 5800+ RPM |

#### Key Technical Capabilities of Better Auto
* **Preemptive Jump:** If the CPU temperature is only 52°C, but you launch a compilation that spikes CPU usage to 75%, Better Auto does not wait for 75°C temperatures. It proactively elevates fans to **Level 6**, keeping the heatsink cool before thermal saturation occurs.
* **Asymmetrical Hysteresis:** Fan ramp-up is instant. Fan ramp-down requires temperatures to drop 3°C to 4°C below the trigger threshold (`DOWN_THRESHOLDS`), preventing repetitive fan oscillation.
* **Single-Step Ramp-Down Limiter:** Even when load drops to zero, fans are never dropped by more than 1 level per cycle. This eliminates jarring acoustic drops.
* **Configurable Acoustic Ceiling:** In Balanced mode, users can set a cap (Levels 3 through 8, default Level 5). Better Auto will not exceed this noise level during normal tasks.
* **Emergency Thermal Bypass (\(\ge 88^\circ\text{C}\)):** If unexpected thermal load pushes temperature to 88°C or above, the acoustic ceiling is **instantly bypassed** and fans jump to Level 8 (100% Turbo) to protect hardware integrity.
* **Custom Minimum RPM (Default 2600 RPM):** Keeps airflow moving at an inaudible speed, avoiding high-pitched start-stop fan motor wear. Configurable between 2000 RPM and 3500 RPM in GUI Settings.
* **10-Second EC Stagger Protection:** HP Victus Embedded Controllers (EC) can freeze if both Fan 1 (CPU) and Fan 2 (GPU) registers are written simultaneously. Victus Max writes Fan 1, waits asynchronously for 10 seconds, then writes Fan 2.
* **90-Second Watchdog:** HP BIOS attempts to regain fan control every 60–90 seconds. Victus Max automatically re-asserts mode control every 80–90 seconds in the background.

---

### 2. Auto (`auto`)
* **Engine:** Stock HP Embedded Controller / BIOS thermal table.
* **Mechanism:** Sets `pwm1_enable` to `2` (OEM automatic mode).
* **Behavior:** Relies purely on the factory thermal curve programmed into your motherboard's firmware.
* **Pros:** Zero software overhead, 100% factory behavior.
* **Cons:** Prone to thermal lag; fans tend to stay quiet until temperatures cross 75°C, then spike abruptly.

---

### 3. Max Fan Boost (`max`)
* **Engine:** Victus Max Full Turbo Override.
* **Mechanism:** Sets `pwm1_enable` to `0` and writes `255` (100% duty cycle) to hardware registers.
* **Behavior:** Both CPU and GPU fans are locked to their maximum hardware RPM (typically 5200–5900 RPM depending on the model).
* **Pros:** Maximum possible thermal headroom. Ideal for intense gaming sessions, heavy renders, or benchmarking.
* **Cons:** High acoustic noise; not suitable for quiet environments.

---

### 4. Custom Curve (`custom`)
* **Engine:** Victus Max Spline & Multi-point Interpolator.
* **Mechanism:** Reads custom temperature-to-speed percentage points defined by the user in `~/.config/victus-max/fan_presets.json`.
* **Behavior:** Linearly interpolates fan RPM between your custom control points (e.g., 40°C \(\to\) 25%, 60°C \(\to\) 45%, 75°C \(\to\) 70%, 85°C \(\to\) 100%).
* **Pros:** Full creative freedom for advanced enthusiasts who prefer tailor-made curves.
* **Cons:** Requires manual tuning to avoid thermal throttling.

---

### 5. Hardware EC Mode (`ec`)
* **Engine:** Raw Hardware Embedded Controller.
* **Mechanism:** Completely disengages software fan intervention.
* **Behavior:** Signals the BIOS that the operating system is relinquishing fan control. Due to HP EC hardware watchdog timings, it may take 60 to 120 seconds for the BIOS to fully recapture fan modulation.
* **When to use:** Crucial safe fallback for unverified motherboard revisions where WMI `0x2E` register writes are rejected by BIOS.

---

## Power & Fan Synchronization Matrix

Victus Max automatically harmonizes your chosen power profile with your fan cooling rules:

```
┌─────────────────┬────────────────────┬──────────────────┬─────────────────┬────────────────────┐
│ Power Profile   │ CPU / GPU Power    │ Better Auto Cap  │ Min Fan RPM     │ 88°C Emergency Cap │
├─────────────────┼────────────────────┼──────────────────┼─────────────────┼────────────────────┤
│ Quiet / Eco     │ Capped Low (15-25W)│ Level 3 (~3300)  │ User (Def 2600) │ Bypasses to Lvl 8  │
│ Balanced        │ Nominal (45W/80W)  │ Level 5 (~4100)  │ User (Def 2600) │ Bypasses to Lvl 8  │
│ Performance     │ Max Boost (PL1/PL2)│ Level 8 (No Cap) │ User (Def 2600) │ Active (Lvl 8)     │
└─────────────────┴────────────────────┴──────────────────┴─────────────────┴────────────────────┘
```

> **Note:** When switching power profiles, Better Auto **instantly resets** its hysteresis window and cooldown timers. You never have to wait for old cooldown timers when moving from Balanced to Performance.

---

## Real-World Scenarios & Walkthroughs

### Scenario 1: Daily Productivity & Office Work
**Workload:** Web browsing (Firefox/Chrome with 20+ tabs), writing documents, Slack, VS Code, watching YouTube or Twitch.

* **Recommended Settings:**
  - **Power Mode:** `Balanced`
  - **Fan Mode:** `Better Auto` (Acoustic Ceiling: Level 4 or 5; Min RPM: 2600 RPM)
* **What Happens Under the Hood:**
  1. The CPU runs at nominal clock speeds. The discrete NVIDIA GPU stays suspended in D3cold power-saving mode, saving battery and keeping temperatures low.
  2. While typing or reading, temperature rests around 44°C–48°C. Better Auto maintains Level 1 (2600 RPM)—a gentle, virtually silent breeze that keeps internal chassis air circulating.
  3. When loading a heavy website or running a fast script, CPU usage briefly spikes to 65%. Better Auto catches the spike in `/proc/stat` and elevates fans to Level 4 (~3700 RPM).
  4. The acoustic ceiling caps fan speed at Level 5, ensuring you never hear sudden loud fan whines during regular office work.
  5. Once the page finishes loading, the single-step ramp-down limiter smoothly glides fans back down without sudden pitch shifts.

---

### Scenario 2: High-End & Competitive Gaming
**Workload:** Playing *Cyberpunk 2077*, *Counter-Strike 2*, *Valorant*, *Apex Legends*, or *Helldivers 2*.

* **Recommended Settings:**
  - **Power Mode:** `Performance`
  - **Fan Mode:** `Better Auto` or `Max Fan Boost`
* **What Happens Under the Hood:**
  1. Setting `Performance` mode commands the HP WMI controller and ACPI platform profile to allocate full wattage.
  2. The NVIDIA GPU receives maximum TGP plus Dynamic Boost (`ppab = 1`), unlocking highest graphics clock rates and frame rates.
  3. Better Auto automatically detects `Performance` mode and unlocks its acoustic ceiling to **Level 8**.
  4. As soon as the game 3D engine initializes, workload usage jumps above 80%. Better Auto preemptively scales fans to Level 7 / Level 8 *before* temperatures reach 80°C.
  5. Because cooling is preemptive rather than reactive, your GPU stays below 75°C and CPU below 85°C, preventing thermal throttling drops (stuttering) during crucial firefights.

---

### Scenario 3: Library, Classroom, or Late-Night Quiet Study
**Workload:** Reading PDFs, coding quietly in a lecture hall or library, working in bed late at night.

* **Recommended Settings:**
  - **Power Mode:** `Quiet / Eco`
  - **Fan Mode:** `Better Auto` (or `Auto`)
* **What Happens Under the Hood:**
  1. `Quiet` mode instructs the kernel power governor and Intel RAPL / AMD RyzenAdj to limit processor draw to conservative thresholds.
  2. The GPU remains in deep sleep.
  3. Better Auto clamps the acoustic ceiling to **Level 3** (~3300 RPM maximum), regardless of your previous settings.
  4. The laptop remains whisper-quiet. The fans never rev up past a faint murmur.
  5. **Safety Guarantee:** If a runaway background process (e.g., an indexing loop) causes temperatures to reach 88°C, Better Auto's emergency bypass activates, spins the fans up to protect the motherboard, and drops back down once cooled.

---

### Scenario 4: Heavy Code Compilation & 3D Rendering
**Workload:** Compiling large projects (`cargo build --release`, Linux kernel builds), Blender rendering, DaVinci Resolve 4K video exports.

* **Recommended Settings:**
  - **Power Mode:** `Performance`
  - **Fan Mode:** `Better Auto` (or `Custom Curve`)
* **What Happens Under the Hood:**
  1. All CPU cores are pinned at 100% utilization.
  2. Better Auto detects 100% CPU usage within its 1-second monitoring window and immediately engages Level 8 cooling.
  3. With maximum thermal dissipation active from the start, the CPU can sustain high multi-core boost clocks for several minutes without prematurely hitting thermal trip points.
  4. Build completion times are significantly faster compared to stock BIOS reactive profiles.

---

### Scenario 5: Periodic Heatsink Dust Cleaning
**Workload:** Monthly maintenance to purge dust bunnies from heatsink exhaust fins.

* **Recommended Action:**
  - Trigger **Clean Fans** via GUI Maintenance or terminal command:
    ```bash
    victus-max-cli system clean-fans
    ```
* **What Happens Under the Hood:**
  1. The daemon initiates a pulsed fan ritual.
  2. Fans are driven to 100% maximum boost for rapid air velocity, followed by deliberate RPM shifts.
  3. Air pressure dislodges accumulated lint and dust from the rear and side exhaust fins.
  4. The daemon automatically restores your previous cooling mode (e.g., `Better Auto`) upon completion.

---

## Quick Reference Cheat Sheet

| Use Case | Suggested Power Mode | Suggested Fan Mode | Acoustic Experience | Thermal Priority |
|:---|:---:|:---:|:---:|:---:|
| **Web & Office** | Balanced | Better Auto | Whisper-Quiet | Moderate |
| **Gaming (AAA / Esports)** | Performance | Better Auto | Audible Turbo | Maximum Headroom |
| **Coding & Compiling** | Performance | Better Auto | High on Build | Clock Stability |
| **Library / Class** | Quiet / Eco | Better Auto | Inaudible | Noise Suppression |
| **Benchmarking** | Performance | Max Boost | Loud (100% Turbo) | Absolute Peak |
| **Testing Unverified Board**| Balanced | Hardware EC | Factory Stock | OEM Firmware Managed |

---

## Control Interfaces

Victus Max gives you complete freedom to toggle any mode instantly using your preferred method:

### 1. In-Game HUD Overlay (Shift + F2)
Press `Shift + F2` anywhere to toggle the floating glassmorphism HUD:
- **Fan Modes:** `[Q]` Better Auto &nbsp;•&nbsp; `[W]` Auto &nbsp;•&nbsp; `[E]` Max Boost &nbsp;•&nbsp; `[R]` Custom
- **Power Modes:** `[1]` Quiet &nbsp;•&nbsp; `[2]` Balanced &nbsp;•&nbsp; `[3]` Performance

### 2. System Tray Icon
Right-click the Victus Max tray icon in your system panel:
- Select **Fan Modes** \(\to\) check **Better Auto**, **Auto**, **Max**, or **EC**.
- Select **Power Profiles** \(\to\) choose **Performance**, **Balanced**, or **Quiet**.

### 3. Command Line Interface (`victus-max-cli`)
```bash
# View live telemetry, active power profile, and fan RPMs
victus-max-cli fetch
victus-max-cli fan info

# Switch Fan Modes
victus-max-cli fan set-mode better-auto
victus-max-cli fan set-mode max
victus-max-cli fan set-mode auto

# Configure Better Auto Acoustic Limits
victus-max-cli fan set-min-rpm 2600     # Range: 2000 - 3500 RPM
victus-max-cli fan set-ceiling 5        # Range: Level 3 to Level 8

# Switch Power Modes
victus-max-cli power set-profile performance
victus-max-cli power set-profile balanced
victus-max-cli power set-profile quiet

# Run Fan Cleaning Ritual
victus-max-cli system clean-fans
```

### 4. Graphical Control Center (`victus-max`)
Launch **Victus Max** from your applications menu to configure fan curves visually, adjust keyboard RGB lighting, manage app profiles, and fine-tune acoustic ceilings.
