---
type: Guide
title: Usage
description: How to run, control, and monitor the jitter filter.
tags: [usage, tray-icon, controls]
timestamp: 2026-06-25T17:00:00Z
---

# Usage

## Starting the app

Double-click `jitter-filter.exe`. No window appears — only a tray icon in the notification area (near the clock).

## Tray icon controls

| Action | Result |
|---|---|
| **Left double-click** | Shows a balloon tip: "Active — filtering trackpad jitter" |
| **Right-click** | Opens context menu with two options |
| **Right-click → Exit** | Stops the filter and removes the tray icon |
| **Right-click → Buy me a coffee ☕** | Opens [buymeacoffee.com/bartosz.janiak](https://buymeacoffee.com/bartosz.janiak) in the default browser |

## Verifying it works

1. Run the app (tray icon appears).
2. Touch the touchpad lightly without deliberate movement — the cursor should stay still (jitter suppressed).
3. Move your finger deliberately — cursor should respond normally, with subtle amplification to compensate for filtered micro-movements.

## Stopping the app

- Right-click tray → Exit, or
- Kill `jitter-filter.exe` in Task Manager.

## Tray icon

The app looks for `jitter.png` in the same directory as the executable. If found, it uses that image as the tray icon. If missing, it falls back to the default Windows application icon.

## Logging

The app has no logging in release mode (no console, no file output). To verify operation, observe cursor behavior or check that the process is running in Task Manager.