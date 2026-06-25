# Touchpad Jitter Filter

Windows tray app that filters trackpad jitter on devices like the Chuwi Minibook X.

Low-level mouse hook (`WH_MOUSE_LL`) detects and suppresses small, rapid cursor movements typical of
cheap touchpad hardware, while amplifying deliberate motion to avoid the "stuck cursor" feeling.

## Usage

Run `jitter-filter.exe`. A tray icon appears in the notification area.

- **Right-click** → **Exit** to quit.
- **Right-click** → **Buy me a coffee ☕** to show support.
- **Double-click** the tray icon to show a status balloon.

To autostart with Windows, place a shortcut to the `.exe` in `shell:startup`.

## How it works

- Intercepts `WM_MOUSEMOVE` via `WH_MOUSE_LL`.
- Blocks moves ≤12 px within 25 ms (jitter).
- Single-instance enforced via a named mutex (`Local\JitterFilterSingleton`).
- No console window, no runtime dependencies — pure Rust, single `.exe` (~129 KB).

## Build

```bash
cargo build --release
```