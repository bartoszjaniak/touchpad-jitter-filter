---
type: Guide
title: Installation
description: How to download, build, and set up the jitter filter on Windows.
tags: [installation, setup, autostart]
timestamp: 2026-06-25T17:00:00Z
---

# Installation

## Option 1: Download pre-built binary

1. Go to the [Releases page](https://github.com/bartoszjaniak/touchpad-jitter-filter/releases).
2. Download `jitter-filter.exe` from the latest release.
3. (Optional) Download `jitter.png` for the tray icon and place it next to the `.exe`.

## Option 2: Build from source

### Prerequisites

- [Rust](https://rustup.rs/) (edition 2024, MSRV: 1.85+)
- Windows 10 or 11 (64-bit)

### Steps

```powershell
git clone https://github.com/bartoszjaniak/touchpad-jitter-filter.git
cd touchpad-jitter-filter
cargo build --release
```

The binary is at `target\release\jitter-filter.exe` (~130 KB).

## Autostart (run on login)

### Via Startup folder

1. Press `Win + R`, type `shell:startup`, press Enter.
2. Create a shortcut to `jitter-filter.exe` in the opened folder.
3. (Optional) Place `jitter.png` in the same folder as the shortcut for the tray icon.

### Via Task Scheduler (alternative)

1. Open Task Scheduler.
2. Create a new task: trigger "At logon", action "Start program" → `jitter-filter.exe`.
3. Set "Run whether user is logged on or not" for headless operation.

## Uninstall

1. **Exit the app**: right-click tray icon → Exit.
2. **Remove from autostart**: delete the shortcut from `shell:startup`.
3. **Delete the binary**: remove `jitter-filter.exe` and any accompanying `jitter.png`.