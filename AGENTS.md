# AGENTS.md

Guidance for coding agents working on Chronote.

## What this is

Chronote is a desktop app (Rust + [iced](https://iced.rs) 0.14) for **study
timelines**: a medical student (or anyone) builds a timeline in their own unit
of time, places instant events (points) and lasting events (bars) on it, and
attaches study notes and PDF/DOCX files. It must run on **Linux and Windows
10/11**.

The **Embryo** timeline is only a built-in demo (animated prenatal
development). Do not treat it as the core of the app.

## Layout

| Path | Role |
| --- | --- |
| `src/main.rs` | Startup: options, renderer choice, logging, fonts, window |
| `src/app.rs` | Root state; switches between menu, demo and editor screens |
| `src/menu.rs` | Startup list of timelines, sorting, font setting |
| `src/editor.rs`, `src/track.rs` | Timeline editor and its canvas |
| `src/project.rs` | Data model (`Library`, `Project`, `Event`) and `Store` (JSON + files) |
| `src/startup.rs` | Renderer selection and recovery from failed GPU starts |
| `src/logging.rs`, `src/crash.rs` | Log file and crash dialog |
| `src/ui.rs`, `src/theme.rs`, `src/anim.rs` | Shared widgets, fonts, colours, easing |
| `src/demo.rs`, `src/data.rs`, `src/fetus.rs`, `src/timeline.rs` | Embryo demo only |
| `assets/` | Bundled Inter font (OFL) and the app icon (SVG source, PNG, ICO) |
| `build.rs` | Embeds the icon and version info into the Windows `.exe` |

`examples/` and `scripts/` are git-ignored, for local-only helpers.

## Commands

```sh
cargo run                                   # run the app
cargo test --all-targets                    # tests
cargo clippy --all-targets -- -D warnings   # lints (pedantic, must be clean)
cargo fmt --all                             # formatting
prek install --install-hooks                # once per clone
prek run --all-files                        # every pre-commit check
```

## Rules

- **Commits:** small and per logical change, using Conventional Commits
  (`feat(scope): …`, `fix: …`, `refactor!: …` with a `BREAKING CHANGE:`
  footer). `cog verify` runs as a commit-msg hook. Never skip hooks.
- **Before committing:** fmt, clippy (`-D warnings`, pedantic), tests and
  rust-analyzer diagnostics must all pass. The hooks enforce this.
- **prek caveat:** hooks run on the working tree. prek stashes unstaged edits
  to tracked files but leaves untracked files in place, so an unrelated new
  file can break (or be checked with) an unrelated commit. Stage everything a
  commit needs, and nothing else.
- **No `unsafe`.** The crate denies it; the only exception is the documented
  `set_var` in `main.rs` that runs before any other thread starts.
- **Lint allowances** live in `Cargo.toml` with a reason; don't add new ones
  without one.

## Conventions and gotchas

- **iced 0.14 is Elm-style:** each screen has `Message`, `update` and `view`.
  Screens return an `Outcome` to the parent instead of reaching into it.
  Subscription `map`/`filter_map` closures must not capture anything (use
  function items).
- **Saving:** every change saves through `App::save()`. When you add fields to
  saved types, use `#[serde(default)]` so existing `library.json` files still
  load. A damaged library is renamed (`Store::quarantine`), never overwritten.
- **Fonts:** use `ui::regular()`, `ui::semibold()`, `ui::bold()`, never
  `Font::DEFAULT`. Canvas `Text` needs `font: ui::regular()` set explicitly.
  The family can only change at startup (custom font setting + restart).
- **Rendering:** MSAA stays off; it made the wgpu renderer drop canvas meshes.
  On Windows only DX12 is used by default. If a start never draws a frame,
  the next start uses the software renderer (`src/startup.rs`).
- **Logging:** use the `log` macros. `info!` for milestones (startup, screens
  opened, files saved or attached), `warn!` for recoverable failures, `error!`
  for crashes. The Windows release build has no console, so `chronote.log` in
  the data directory is the only way to see what happened.
- **Animation** runs on `iced::time::every` (~60 fps) only on the demo
  screen. Don't add per-frame subscriptions to other screens.
- **Data directory:** `directories` `ProjectDirs("dev", "chronote",
  "chronote")`, overridable with `CHRONOTE_DATA_DIR` (handy in tests).

## Releasing

1. `cog bump --version X.Y.Z[-pre]`. This writes `Cargo.toml`, the
   CHANGELOG, a `chore(version)` commit and a `vX.Y.Z` tag.
2. `git push origin main vX.Y.Z`.
3. `gh release create vX.Y.Z --prerelease --verify-tag --notes-file …`.
   Publishing triggers `.github/workflows/release.yml`, which builds and
   attaches the Linux `.tar.gz` and the Windows `.exe`.
