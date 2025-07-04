# Changelog

<!-- markdownlint-configure-file { "no-duplicate-heading": { "siblings_only": true } } -->

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

<!--
Possible types: Added, Changed, Deprecated, Removed, Fixed, Security.

When releasing a new version:
* Update links at the bottom
-->

## [Unreleased]

### Added

- User scripts can now wrap elements in a marquee container with `cso2.marquee.wrap(element)`. See [Album Line](https://currentsong.nerixyz.de/Customization/Theming/Examples/#album-line) for an example.
- Added the `use-raw-data` query parameter for the overlay. If it's present in the URL, no cleanup will be performed on the client (example: `http://localhost:48457/?use-raw-data`).
- Linux: The dbus adapter now automatically discovers all `org.mpris.MediaPlayer2.*` services. If you previously ran an instance, you need to update your config and set `modules.dbus.destinations` to `["org.mpris.MediaPlayer2.*"]`.
- Windows: The GSMTC filters now accept a `regex`, which can be case-insensitive. This is used in the default configuration now. See [Configuration](https://currentsong.nerixyz.de/Configuration) for the current default config.

### Fixed

- Linux: Local images (`file://`) are now loaded correctly.
- Windows: The default config now excludes Chrome and Firefox on Windows 11 correctly.

## [v0.1.0-alpha.13] - 2024-06-22

### Fixed

- Custom hosts now resolve links correctly.

## [v0.1.0-alpha.12] - 2024-06-22

### Fixed

- Custom hosts now work correctly with the overlay client and extension.

## [v0.1.0-alpha.11] - 2024-06-21

### Added

- Added support for players implementing the MPRIS D-Bus interface on Unix.
- Added center positioning (`pos=c`, `pos=vcenter`, `pos=hcenter`, [documentation](https://currentsong.nerixyz.de/Customization/Theming/#position)).
- Added [`server.bind`](https://currentsong.nerixyz.de/Configuration/#bind) to specify multiple addresses the application should bind to/listen on.

### Changed

- Adjusted the config location search. This is compatible with the previous behavior.
- If an invalid config is encountered on Windows, the user is now shown a dialog with the error and the options "Cancel", "Try Again" (re-read the config), and "Continue" (replace the config with the default one).

### Fixed

- Removed shadow in [transparent example](https://currentsong.nerixyz.de/Customization/Theming/Examples/#transparent-background).

## [v0.1.0-alpha.10] - 2023-09-01

### Added

- The overlay can be positioned through URL parameters. For example `localhost:48457/?pos=br` to position the overlay in the bottom right corner ([documentation](https://currentsong.nerixyz.de/Customization/Theming/#position)).
- URL parameters (query and hash) are now reflected in the DOM on the `html` element to allow easier customization ([documentation](https://currentsong.nerixyz.de/Customization/Theming/#attributes)).

### Fixed

- Missing user-scripts would break the overlay in dev mode (`pnpm dev`).
- The marquee effect would not work in Chromium < 103.

## [v0.1.0-alpha.9] - 2023-08-04

### Added

- Metadata for custom themes through CSS variables ([documentation](https://currentsong.nerixyz.de/Customization/Theming/#css-classes)).
- User-scripts (`user.js`) are now supported ([documentation](https://currentsong.nerixyz.de/Customization/User%20Scripts/)).
- Text that overflows the container now has a marquee effect (like on Spotify).
- Documentation is now provided at [currentsong.nerixyz.de](https://currentsong.nerixyz.de).

### Removed

- Auto-start by default is now disabled.

[unreleased]: https://github.com/Nerixyz/current-song2/compare/v0.1.0-alpha.10...HEAD
[v0.1.0-alpha.13]: https://github.com/Nerixyz/current-song2/compare/v0.1.0-alpha.12...v0.1.0-alpha.13
[v0.1.0-alpha.12]: https://github.com/Nerixyz/current-song2/compare/v0.1.0-alpha.11...v0.1.0-alpha.12
[v0.1.0-alpha.11]: https://github.com/Nerixyz/current-song2/compare/v0.1.0-alpha.10...v0.1.0-alpha.11
[v0.1.0-alpha.10]: https://github.com/Nerixyz/current-song2/compare/v0.1.0-alpha.9...v0.1.0-alpha.10
[v0.1.0-alpha.9]: https://github.com/Nerixyz/current-song2/compare/v0.1.0-alpha.8...v0.1.0-alpha.9
