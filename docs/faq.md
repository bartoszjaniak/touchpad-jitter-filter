---
type: FAQ
title: Frequently Asked Questions
description: Troubleshooting, tuning, and known issues.
tags: [faq, troubleshooting, tuning]
timestamp: 2026-06-25T17:00:00Z
---

# FAQ

## The cursor still jitters

Increase `THRESHOLD` in `src/main.rs`:

```rust
const THRESHOLD: i32 = 16;  // try 14, 16, 20
```

Then rebuild: `cargo build --release`.

## The cursor feels sluggish / stuck in honey

Decrease `THRESHOLD` or increase `AMPLIFY`:

```rust
const THRESHOLD: i32 = 8;    // less filtering
const AMPLIFY: i32 = 3;      // stronger compensation
```

## The cursor jumps when I rest my finger

Increase `MIN_DIR_CHANGES` — requires more direction reversals before confirming jitter:

```rust
const MIN_DIR_CHANGES: u32 = 5;  // default is 3
```

## The tray icon is missing

- The icon is loaded from `jitter.png` next to the `.exe`. Ensure the file exists.
- Windows may hide the icon in the overflow area. Click the arrow `^` in the notification area and drag "Jitter Filter" to the visible area.
- Restart the app.

## Two instances are running

The app uses a named mutex `Local\JitterFilterSingleton` to enforce single-instance. If you see two, kill both in Task Manager and restart one.

## How do I update?

1. Download the new `jitter-filter.exe` from [Releases](https://github.com/bartoszjaniak/touchpad-jitter-filter/releases).
2. Exit the running app (right-click → Exit).
3. Replace the old `.exe` with the new one.
4. Restart.

## Does it work with external mice?

Yes, the hook intercepts all mouse movements. External mice may trigger the filter if they produce small, rapid movements, but typical mouse movements exceed the threshold and pass through normally.

## Is it safe?

The hook runs in user space, does not modify system files, does not persist data, does not communicate over the network (except opening the coffee link in your browser on explicit click). Source code is fully available for review.