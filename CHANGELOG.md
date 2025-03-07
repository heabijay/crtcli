# crtcli Changelog

## [0.1.2](https://github.com/heabijay/crtcli/releases/tag/v0.1.2) (2025-01-23)

### Added

 - Added `--compile-package | -c` option to the `app pkg install` and `app pkg push` commands ([#9](https://github.com/heabijay/crtcli/pull/9))

 - Added `--check` option to the `pkg apply` command ([#10](https://github.com/heabijay/crtcli/pull/10))

### Changed

 - Simplified source and destination arguments for the `app pkg push` and `app pkg pull` commands ([#8](https://github.com/heabijay/crtcli/pull/8))

 - Improved sorting transform of data values in `Data/**/data.json` and `Data/**/Localization/data.*.json` files ([#11](https://github.com/heabijay/crtcli/pull/11))


## [0.1.1](https://github.com/heabijay/crtcli/releases/tag/v0.1.1) (2025-01-09)

### Added

 - Installation scripts for Linux, macOS, and Windows ([#1](https://github.com/heabijay/crtcli/pull/1))

 - Implemented progress bars (spinners) for long-running commands ([#3](https://github.com/heabijay/crtcli/pull/3))

 - Enabled support for multiple packages in the `app pkg push` command ([#4](https://github.com/heabijay/crtcli/pull/4))

 - Enabled support for multiple packages in the `pkg pack` command ([#6](https://github.com/heabijay/crtcli/pull/6))

### Changed

 - Improved the log watcher during package installation (`app pkg install`, `app pkg push`) to create a new polling session when Creatio is running on .NET Framework (IIS) ([#5](https://github.com/heabijay/crtcli/pull/5))

 - Improved colored output messages ([#2](https://github.com/heabijay/crtcli/pull/2))

 - Changed optimization level to maximize performance

### Fixed

 - Fixed an issue where command-line options were not taking precedence over the `package.crtcli.toml` apply configuration ([#2](https://github.com/heabijay/crtcli/pull/2))


## [0.1.0](https://github.com/heabijay/crtcli/releases/tag/v0.1.0) (2025-01-03)

### Added

 - Initial Release 🎉