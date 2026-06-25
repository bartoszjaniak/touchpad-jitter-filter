---
type: Design Document
title: Architecture
description: Design of the jitter filter — low-level mouse hook, direction-change detection, accumulation, and amplification.
tags: [architecture, algorithm, design]
timestamp: 2026-06-25T17:00:00Z
---

# Architecture

## Overview

The application installs a system-wide low-level mouse hook (`WH_MOUSE_LL`) that intercepts all `WM_MOUSEMOVE` messages before they reach any application. The hook procedure analyses each movement event and decides whether to block it (suppress jitter) or let it pass through.

## Algorithm

### State machine

Each `WM_MOUSEMOVE` event is processed through these stages:

1. **First event** — Store cursor position as anchor, block the event.
2. **Time window check** — If more than 25 ms have elapsed since the last event, reset all state and pass through.
3. **Size check** — If movement ≤ `THRESHOLD` (12 px) in both axes, proceed to direction analysis. Otherwise, reset state and pass through.
4. **Direction analysis** — Compare sign of current delta with last delta:
   - **Direction change** (+x → -x or +y → -y) → increment direction-change counter, accumulate displacement, block event.
   - **Same direction** → if in jitter mode, continue blocking (no flush mid-jitter).
5. **Jitter activation** — When direction-change counter reaches `MIN_DIR_CHANGES` (3), enter jitter mode.
6. **Jitter exit** — When a gap > 25 ms occurs with accumulated displacement, flush accumulated value × `AMPLIFY` (2×) via `SendInput` relative move, then exit jitter mode and reset.
7. **Big movement** — Any movement exceeding `THRESHOLD` resets all state and passes through immediately.

### Diagram

```
Mouse event → within THRESHOLD? → No → Reset state → Pass through
                 ↓ Yes
          within 25ms? → No → Reset state → Pass through
                 ↓ Yes
          Direction changed? → Yes → Increment counter, accumulate, block
                 ↓ No
          IN_JITTER? → Yes → Block (no flush)
                 ↓ No
          Pass through (normal movement)
```

### Why direction-change detection?

Touchpad jitter manifests as rapid, small-amplitude movements with frequent direction reversals — the finger micro-vibrates on the sensor. Deliberate movement, even slow, maintains a consistent direction. By keying on direction changes rather than just amplitude, the filter distinguishes jitter from intentional slow movement.

### Why accumulation + amplification?

Blocking jitter means some cursor travel is lost. The accumulator tracks this lost displacement. When jitter subsides and movement resumes, a relative `SendInput` sends the accumulated displacement × `AMPLIFY` to restore cursor responsiveness without causing a feedback loop (unlike `SetCursorPos` which generates synthetic `WM_MOUSEMOVE` events).

## Constants

| Constant | Default | Purpose |
|---|---|---|
| `THRESHOLD` | 12 px | Maximum displacement classified as "small" |
| `AMPLIFY` | 2× | Multiplier for accumulated displacement flush |
| `MIN_DIR_CHANGES` | 3 | Direction reversals before jitter is confirmed |
| Time window | 25 ms | Maximum gap between events considered "continuous" |

## Tray icon

A hidden window receives `Shell_NotifyIconW` callbacks. Right-click opens a popup menu with Exit and Buy Me a Coffee. The icon is loaded from `jitter.png` adjacent to the executable, falling back to the default application icon.

## Process model

- Single-instance via named mutex `Local\JitterFilterSingleton`
- No console window (`#![windows_subsystem = "windows"]`)
- Runs until Exit is selected or process is killed

## Dependencies

- `windows-sys` 0.59 (Win32 FFI bindings)
- No runtime dependencies — statically linked, single .exe