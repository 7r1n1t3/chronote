# Chronote

Animated study timelines with notes, built with Rust and [iced](https://iced.rs).
Runs on Linux and Windows.

The app opens on a menu of timelines:

- **Embryo (demo):** a week-by-week animated journey through human prenatal
  development. The procedural illustration morphs from egg to cell division,
  curled embryo and growing fetus, with a heartbeat at an age-appropriate rate.
  Each week shows its length, weight, a size comparison and key milestones.
- **Your own timelines:** build a timeline for anything you study (heart
  development, the clinical course of sepsis, a drug's pharmacokinetics...)
  with your own unit of time (weeks, days, hours...).
  - **Instant** events show as points, **lasting** events as continuous bars.
    Overlapping events are arranged into lanes automatically.
  - Each event has optional **study notes** and **PDF / DOCX attachments**
    (pick files or drop them onto the window, then open them from the app).
  - Everything saves automatically.

The menu can be sorted by creation date, alphabetically or in your own
manual order.
From the menu you can also pick a custom font (any TTF/OTF file); it applies
after a restart, and *Reset* goes back to the bundled Inter font.

> The demo shows approximate averages for education only, not medical advice.

## Usage

```sh
cargo run --release               # opens the menu
cargo run --release -- --software # force CPU rendering (see below)
```

Demo shortcuts: `←` / `→` (or scroll) to change week, `Space` to play or
pause, `Home` / `End` to jump to the first or last week.

### Where data is stored

| OS      | Location                                   |
| ------- | ------------------------------------------ |
| Linux   | `~/.local/share/chronote/`                 |
| Windows | `%APPDATA%\chronote\chronote\data\`        |

Set `CHRONOTE_DATA_DIR` to use a different folder. Attachments are copied into
`attachments/` there, so a timeline stays complete even if the originals move.
A damaged `library.json` is renamed (not overwritten), and a fresh library is
started.

If the app ever fails to start or crashes, it shows an error dialog and writes
the details to `crash.log` in the same folder; please include it in bug reports.

Every run also writes `chronote.log` there (the previous run is kept as
`chronote.log.old`). It records the OS (including whether it is Windows 11),
the graphics adapter and backend iced picked, window size and scale factor,
the first rendered frame, and warnings such as failed saves or attachments.
Set `CHRONOTE_LOG=debug` for more detail.

### If the app does not open

Some graphics drivers, or overlays that hook into them, hang while the GPU
renderer starts. Chronote notices when a start never drew its first frame and
uses the software renderer from then on, saying so on the menu. You can also
choose yourself:

```sh
chronote --software   # always render on the CPU for this run
chronote --gpu        # go back to the GPU after an automatic switch
```

On Windows only Direct3D 12 is used by default; set `WGPU_BACKEND` (for
example `vulkan`) to override it. The log shows which renderer was picked.

## Development

Requirements: a stable Rust toolchain, [prek](https://github.com/j178/prek),
[cocogitto](https://docs.cocogitto.io) (`cog`), `typos` and `rust-analyzer`.

```sh
prek install --install-hooks   # pre-commit + commit-msg hooks
cog commit feat "add something" # or any Conventional Commit message
```

The hooks run whitespace/TOML/YAML checks, `typos`, `cargo fmt`,
`cargo clippy -D warnings` (pedantic), `cargo test`, `rust-analyzer`
diagnostics, and `cog verify` on the commit message. CI repeats the checks on
Linux and Windows.

`examples/` and `scripts/` are git-ignored, for local-only helpers (such
as screenshot tools).

### Known limitations

- MSAA is turned off: with it enabled, iced 0.14's wgpu renderer dropped
  canvas shapes on the development machine, so curved edges look slightly
  sharper than they could.

## License

Chronote is licensed under the [Apache License, Version 2.0](LICENSE).

The bundled [Inter](https://github.com/rsms/inter) font is © The Inter Project
Authors and licensed under the [SIL Open Font License 1.1](assets/fonts/OFL.txt).
