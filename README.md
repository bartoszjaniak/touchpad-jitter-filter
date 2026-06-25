# Touchpad Jitter Filter

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)
![Platform: Windows](https://img.shields.io/badge/platform-Windows%2010%2F11-blue)
![Size: ~130 KB](https://img.shields.io/badge/size-%7E130%20KB-brightgreen)

A Windows system-tray application that filters touchpad jitter using a low-level mouse hook. Designed for devices with noisy touchpad hardware — such as the Chuwi Minibook X — where built-in Windows smoothing is insufficient.

## Features

- **Direction-change detection** — distinguishes jitter (rapid back-and-forth micro-movements) from deliberate slow movement.
- **Adaptive amplification** — blocked jitter displacement is accumulated and amplified when movement resumes, preventing the "stuck cursor" feeling.
- **Tray icon** — right-click menu with Exit and Buy Me a Coffee.
- **Single instance** — multiple launches reuse the running instance.
- **No runtime dependencies** — static Rust binary, just one `.exe`.

## Quick start

1. Download `jitter-filter.exe` from [Releases](https://github.com/bartoszjaniak/touchpad-jitter-filter/releases).
2. Run it. A tray icon appears in the notification area.
3. Done — jitter is filtered immediately.

## Installation

See [Installation guide](docs/installation.md) for:
- Running from source (`cargo build --release`)
- Autostart setup (Startup folder / Task Scheduler)

## Usage

| Action | Result |
|---|---|
| Run the `.exe` | Tray icon appears, filtering starts |
| Right-click tray | Menu: Exit / Buy Me a Coffee |
| Double-click tray | Status balloon |

Full usage guide: [docs/usage.md](docs/usage.md)

## Autostart

Add the app to Windows startup so it runs automatically on login:

1. Press **`Win + R`**, type `shell:startup`, press Enter.
2. Create a shortcut to `jitter-filter.exe` in the opened folder.
3. (Optional) Place `jitter.ico` next to `jitter-filter.exe` for the tray icon.

See [Installation guide](docs/installation.md#autostart-run-on-login) for alternative methods (Task Scheduler).

## Configuration

All parameters are compile-time constants in [`src/main.rs`](src/main.rs):

| Constant | Default | Purpose |
|---|---|---|
| `THRESHOLD` | 12 px | Max displacement classified as "small" |
| `AMPLIFY` | 2× | Amplification for accumulated displacement |
| `MIN_DIR_CHANGES` | 3 | Direction reversals before confirming jitter |
| Time window | 25 ms | Max gap considered "continuous" |

Adjust and rebuild: `cargo build --release`.

## How it works

1. Installs a `WH_MOUSE_LL` hook that intercepts all `WM_MOUSEMOVE` events.
2. Events within 12 px / 25 ms are analysed for **direction changes**:
   - Frequent reversals (+x → -x → +x) → **jitter** → blocked, displacement accumulated.
   - Consistent direction → **deliberate** → passed through.
   - After jitter, accumulated displacement is flushed at 2× via `SendInput` (relative move, no feedback loop).
3. Big movements or gaps > 25 ms reset the state.

Detailed architecture: [docs/architecture.md](docs/architecture.md)

## Build from source

```powershell
git clone https://github.com/bartoszjaniak/touchpad-jitter-filter.git
cd touchpad-jitter-filter
cargo build --release
```

Requires Rust edition 2024 (MSRV 1.85+).

## Contributing

Issues and pull requests are welcome. See [docs/faq.md](docs/faq.md) for known limitations and tuning advice.

## License

MIT

## Support

If this tool saves you time or frustration, consider buying me a coffee:

[☕ buymeacoffee.com/bartosz.janiak](https://buymeacoffee.com/bartosz.janiak)