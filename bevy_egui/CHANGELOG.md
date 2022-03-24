# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.12.1] - 13-Mar-2022

### Added

- Add a function to get image id ([#80](https://github.com/mvlabat/bevy_egui/pull/80) by @Shatur).

## [0.12.0] - 12-Mar-2022

### Added

- Upgrade Egui to 0.17 ([#78](https://github.com/mvlabat/bevy_egui/pull/78) by @emilk).
- Add side panel example ([#73](https://github.com/mvlabat/bevy_egui/pull/73)).

### Changed

- User texture ids are now tracked internally ([#71](https://github.com/mvlabat/bevy_egui/pull/71)).
  - Instead of using `set_egui_texture`, you can now use `add_image` which returns a texture id itself
  (see the updated [ui](https://github.com/mvlabat/bevy_egui/blob/c611671603a70e5956ba06f77bb94851c7ced659/examples/ui.rs) example).
- Switch to `arboard` for managing clipboard ([#72](https://github.com/mvlabat/bevy_egui/pull/72)).

## [0.11.1] - 4-Feb-2022

### Added

- Add `ctx_for_windows_mut` and `try_ctx_for_windows_mut` for accessing multiple contexts without the `multi_threaded` feature.

## [0.11.0] - 4-Feb-2022

### Changed

- Introduce mutable getters for EguiContext, feature gate immutable ones ([#64](https://github.com/mvlabat/bevy_egui/pull/63)).
  - If you used `bevy_egui` without the `multi_threaded` feature, you'll need to change every `ctx` call to `ctx_mut`.

## [0.10.3] - 29-Jan-2022

### Added

- Feature `multi_threaded`, to avoid using `egui/multi_threaded` ([#63](https://github.com/mvlabat/bevy_egui/pull/63) by @ndarilek).

### Fixed

- WGPU crash on minimizing a window ([#62](https://github.com/mvlabat/bevy_egui/pull/62) by @aevyrie).

## [0.10.2] - 23-Jan-2022

### Added

- Horizontal scroll support (Shift + Mouse Wheel).
- Zoom support (Ctrl/Cmd + Mouse Wheel).

### Fixed

- Change points delta from 24 to 50 for `MouseScrollUnit::Line` event.
- Fix handling of mouse button events for Safari (inputs are no longer ignored).
- Scroll is no longer applied to every Bevy window. 

## [0.10.1] - 16-Jan-2022

### Added

- Headless mode support ([#51](https://github.com/mvlabat/bevy_egui/pull/51) by @Shatur).

### Fixed

- Egui pass now runs after `bevy_ui` ([#53](https://github.com/mvlabat/bevy_egui/pull/53) by @jakobhellermann).

## [0.10.0] - 8-Jan-2022

### Added

- Upgrade Bevy to 0.6 ([#25](https://github.com/mvlabat/bevy_egui/pull/25) by @jakobhellermann).

## [0.9.0] - 1-Jan-2022

### Added

- Upgrade Egui to 0.16 ([#49](https://github.com/mvlabat/bevy_egui/pull/49) by @Meshiest).

## [0.8.0] - 27-Nov-2021

### Added

- Upgrade Egui to 0.15.0 ([#45](https://github.com/mvlabat/bevy_egui/pull/45)).

## [0.7.1] - 06-Oct-2021

### Added

- Add `EguiStartupSystem` system labels.

### Fixed

- Initialize egui contexts during startup (fixes [#41](https://github.com/mvlabat/bevy_egui/issues/41)).

## [0.7.0] - 05-Sep-2021

### Added

- Upgrade Egui to 0.14.0 ([#38](https://github.com/mvlabat/bevy_egui/pull/38)).

## [0.6.2] - 15-Aug-2021

### Fixed

- Fix receiving input when holding a button ([#37](https://github.com/mvlabat/bevy_egui/pull/37)).

## [0.6.1] - 20-Jul-2021

### Fixed

- Fix more edge-cases related to invalid scissors.

## [0.6.0] - 29-Jun-2021

### Added

- Upgrade Egui to 0.13.0.

## [0.5.0] - 22-May-2021

### Added

- Upgrade Egui to 0.12.0.

## [0.4.2] - 03-May-2021

### Added

- Better error message for a missing Egui context ([#24](https://github.com/mvlabat/bevy_egui/pull/24) by @jakobhellermann)
- Add `try_ctx_for_window` function ([#20](https://github.com/mvlabat/bevy_egui/pull/20) by @jakobhellermann)

## [0.4.1] - 24-Apr-2021

### Fixed

- Fix crashes related to invalid scissor or window size ([#18](https://github.com/mvlabat/bevy_egui/pull/18))

## [0.4.0] - 10-Apr-2021

Huge thanks to @jakobhellermann and @Weasy666 for contributing to this release!

### Added

- Implement Egui 0.11.0 support ([#12](https://github.com/mvlabat/bevy_egui/pull/12) by @Weasy666 and @jakobhellermann).
- Implement multiple windows support ([#14](https://github.com/mvlabat/bevy_egui/pull/14) by @jakobhellermann).

## [0.3.0] - 02-Mar-2021

### Added

- Upgrade Egui to 0.10.0.

## [0.2.0] - 08-Feb-2021

### Added

- Implement Egui 0.9.0 support.

## [0.1.3] - 20-Jan-2021

### Fixed

- Fix copying textures to take alignment into account.
- Disable a documentation test.

## [0.1.2] - 18-Jan-2021

### Fixed

- Disable default features for docs.rs to fix the build.

## [0.1.1] - 18-Jan-2021

### Fixed

- Fix compilation errors when no features are set.