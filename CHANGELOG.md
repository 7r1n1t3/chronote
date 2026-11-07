# Changelog
All notable changes to this project will be documented in this file. See [conventional commits](https://www.conventionalcommits.org/) for commit guidelines.

- - -
## v0.1.0-alpha.1 - 2026-11-07
#### Features
- (**settings**) choose a custom font - (579ad3b) - 7r1n1t3, *Claude Opus 5.5*
#### Bug Fixes
- (**ui**) bundle the Inter font so text renders everywhere - (f9fa0bb) - 7r1n1t3, *Claude Opus 5.5*
- report crashes instead of exiting silently - (294d759) - 7r1n1t3, *Claude Opus 5.5*
#### Documentation
- document the font setting and crash log - (5d18717) - 7r1n1t3, *Claude Opus 5.5*
- license the project under Apache 2.0 - (b371517) - 7r1n1t3, *Claude Opus 5.5*

- - -

## v0.1.0-alpha - 2026-11-07
#### Features
- (**app**) wire up the application shell - (bd441e9) - 7r1n1t3, *Claude Opus 5.5*
- (**data**) add week-by-week prenatal development dataset - (6366256) - 7r1n1t3, *Claude Opus 5.5*
- (**editor**) attach PDF and DOCX files to events - (ee272ca) - 7r1n1t3, *Claude Opus 5.5*
- (**editor**) edit projects on a custom-unit timeline - (4978a8b) - 7r1n1t3, *Claude Opus 5.5*
- <span style="background-color: #d73a49; color: white; padding: 2px 6px; border-radius: 3px; font-weight: bold; font-size: 0.85em;">BREAKING</span>(**menu**) open on a project menu - (e234a37) - 7r1n1t3, *Claude Opus 5.5*
- (**project**) add project model and persistent storage - (8b5bc13) - 7r1n1t3, *Claude Opus 5.5*
- (**ui**) add interactive timeline scrubber - (ed0f34b) - 7r1n1t3, *Claude Opus 5.5*
- (**ui**) add procedural animated fetus illustration - (fbcc3b8) - 7r1n1t3, *Claude Opus 5.5*
- (**ui**) add color palette, theme and animation helpers - (e7cbf5c) - 7r1n1t3, *Claude Opus 5.5*
- present user projects as study timelines - (cb7a91d) - 7r1n1t3, *Claude Opus 5.5*
#### Bug Fixes
- (**data**) use a shorter size comparison for week 2 - (3feaa94) - 7r1n1t3, *Claude Opus 5.5*
- (**fetus**) prevent NaN body width at the end of the spine - (d2f8269) - 7r1n1t3, *Claude Opus 5.5*
- (**hooks**) ignore rust-analyzer inactive_code hints - (dd4a403) - 7r1n1t3, *Claude Opus 5.5*
- (**hooks**) match rust-analyzer diagnostic output format - (8009d4d) - 7r1n1t3, *Claude Opus 5.5*
- (**ui**) stop header buttons from stretching to full width - (3b193f8) - 7r1n1t3, *Claude Opus 5.5*
#### Documentation
- add README - (47d2d15) - 7r1n1t3, *Claude Opus 5.5*
#### Build system
- (**cog**) write the bumped version into Cargo.toml - (f3cc0d7) - 7r1n1t3, *Claude Opus 5.5*
#### Continuous Integration
- build and attach Linux and Windows binaries to releases - (dbf488d) - 7r1n1t3, *Claude Opus 5.5*
- check formatting, lints, tests and builds on Linux and Windows - (f819d7b) - 7r1n1t3, *Claude Opus 5.5*
#### Refactoring
- <span style="background-color: #d73a49; color: white; padding: 2px 6px; border-radius: 3px; font-weight: bold; font-size: 0.85em;">BREAKING</span>rename the application to Chronote - (150c0c4) - 7r1n1t3, *Claude Opus 5.5*
- <span style="background-color: #d73a49; color: white; padding: 2px 6px; border-radius: 3px; font-weight: bold; font-size: 0.85em;">BREAKING</span>move the embryo screen into a demo module - (fc8bd7c) - 7r1n1t3, *Claude Opus 5.5*
#### Miscellaneous Chores
- (**dev**) add renderer screenshot example - (c96ab92) - 7r1n1t3, *Claude Opus 5.5*
- (**version**) v0.1.0-alpha - (ab67eb0) - 7r1n1t3
- scaffold project with iced, cog and prek tooling - (8295a87) - 7r1n1t3, *Claude Opus 5.5*
#### Style
- (**fetus**) refine body proportions and cord routing - (37d0dda) - 7r1n1t3, *Claude Opus 5.5*

- - -

## v0.1.0-alpha - 2026-11-07
#### Features
- (**app**) wire up the application shell - (bd441e9) - 7r1n1t3, *Claude Opus 5.5*
- (**data**) add week-by-week prenatal development dataset - (6366256) - 7r1n1t3, *Claude Opus 5.5*
- (**editor**) attach PDF and DOCX files to events - (ee272ca) - 7r1n1t3, *Claude Opus 5.5*
- (**editor**) edit projects on a custom-unit timeline - (4978a8b) - 7r1n1t3, *Claude Opus 5.5*
- <span style="background-color: #d73a49; color: white; padding: 2px 6px; border-radius: 3px; font-weight: bold; font-size: 0.85em;">BREAKING</span>(**menu**) open on a project menu - (e234a37) - 7r1n1t3, *Claude Opus 5.5*
- (**project**) add project model and persistent storage - (8b5bc13) - 7r1n1t3, *Claude Opus 5.5*
- (**ui**) add interactive timeline scrubber - (ed0f34b) - 7r1n1t3, *Claude Opus 5.5*
- (**ui**) add procedural animated fetus illustration - (fbcc3b8) - 7r1n1t3, *Claude Opus 5.5*
- (**ui**) add color palette, theme and animation helpers - (e7cbf5c) - 7r1n1t3, *Claude Opus 5.5*
- present user projects as study timelines - (cb7a91d) - 7r1n1t3, *Claude Opus 5.5*
#### Bug Fixes
- (**data**) use a shorter size comparison for week 2 - (3feaa94) - 7r1n1t3, *Claude Opus 5.5*
- (**fetus**) prevent NaN body width at the end of the spine - (d2f8269) - 7r1n1t3, *Claude Opus 5.5*
- (**hooks**) ignore rust-analyzer inactive_code hints - (dd4a403) - 7r1n1t3, *Claude Opus 5.5*
- (**hooks**) match rust-analyzer diagnostic output format - (8009d4d) - 7r1n1t3, *Claude Opus 5.5*
- (**ui**) stop header buttons from stretching to full width - (3b193f8) - 7r1n1t3, *Claude Opus 5.5*
#### Documentation
- add README - (47d2d15) - 7r1n1t3, *Claude Opus 5.5*
#### Build system
- (**cog**) write the bumped version into Cargo.toml - (f3cc0d7) - 7r1n1t3, *Claude Opus 5.5*
#### Continuous Integration
- build and attach Linux and Windows binaries to releases - (dbf488d) - 7r1n1t3, *Claude Opus 5.5*
- check formatting, lints, tests and builds on Linux and Windows - (f819d7b) - 7r1n1t3, *Claude Opus 5.5*
#### Refactoring
- <span style="background-color: #d73a49; color: white; padding: 2px 6px; border-radius: 3px; font-weight: bold; font-size: 0.85em;">BREAKING</span>rename the application to Chronote - (150c0c4) - 7r1n1t3, *Claude Opus 5.5*
- <span style="background-color: #d73a49; color: white; padding: 2px 6px; border-radius: 3px; font-weight: bold; font-size: 0.85em;">BREAKING</span>move the embryo screen into a demo module - (fc8bd7c) - 7r1n1t3, *Claude Opus 5.5*
#### Miscellaneous Chores
- (**dev**) add renderer screenshot example - (c96ab92) - 7r1n1t3, *Claude Opus 5.5*
- scaffold project with iced, cog and prek tooling - (8295a87) - 7r1n1t3, *Claude Opus 5.5*
#### Style
- (**fetus**) refine body proportions and cord routing - (37d0dda) - 7r1n1t3, *Claude Opus 5.5*

- - -

Changelog generated by [cocogitto](https://github.com/cocogitto/cocogitto).