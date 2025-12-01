# crtcli Changelog

## [0.3.0](https://github.com/heabijay/crtcli/releases/tag/v0.3.0) (2025-12-01)

### Added

 - Binary data support from stdin for the `app request` command

 - `--watch` flag for the `app install` command to display log updates in real-time

 - Multi-package support for `app pkg compile`, `app pkg fs pull`, `app pkg fs push`, `app pkg lock`, `app pkg unlock`, and `pkg apply` ([#32](https://github.com/heabijay/crtcli/pull/32))

 - Support for multiple package file arguments in the `app pkg install` command ([#32](https://github.com/heabijay/crtcli/pull/32))

 - Alias `ua` for the `pkg unpack-all` command (e.g., `crtcli pkg ua` or `crtcli p ua`) ([#32](https://github.com/heabijay/crtcli/pull/32))

 - `default_app` property in the `.crtcli.toml` configuration file ([#34](https://github.com/heabijay/crtcli/pull/34))

 - New post-transform option to regenerate `.csproj` package references for `pkg apply`-related commands ([#35](https://github.com/heabijay/crtcli/pull/35))

 - Use empty destinations (e.g., `UsrPackage:`) to unpack package to current directory with package name (e.g., `./UsrPackage`) in the `app pkg pull` command

 - New `workspace.crtcli.toml` configuration file designed to manage multiple packages simultaneously ([#36](https://github.com/heabijay/crtcli/pull/36))

 - Smart merge option for `pkg unpack` and `app pkg pull` commands to minimize insignificant schema differences ([#37](https://github.com/heabijay/crtcli/pull/37))

### Changed

 - The `pkg apply` and `pkg pack` commands now default to the current directory if no package folder is specified ([#32](https://github.com/heabijay/crtcli/pull/32))

 - Renamed the legacy `workspace.crtcli.toml` to `.crtcli.toml`. The structure remains the same, with a new optional `root` property ([#33](https://github.com/heabijay/crtcli/pull/33))

 - Replaced the positional "destination" argument with the `--destination` (`-d`) option in the `pkg unpack` command

 - Replaced the `--force-new-session` option with `--clear-session-cache` in the `app` command

### Breaking Change

 - Due to the config file rename ([#33](https://github.com/heabijay/crtcli/pull/33)), existing `workspace.crtcli.toml` files must be renamed to `.crtcli.toml` for a seamless transition


## [0.2.1](https://github.com/heabijay/crtcli/releases/tag/v0.2.1) (2025-10-05)

### Added

 - Added "netframework" as an alias for "net_framework" in the workspace.crtcli.toml file

 - Added new BOM normalization transform for `pkg apply`-related commands ([#26](https://github.com/heabijay/crtcli/pull/26))

 - `app pkg download` can now output package to standard output (stdout) using `--output -` ([#28](https://github.com/heabijay/crtcli/pull/28))

 - The sorting transform in `pkg apply` now supports Resources .xml files ([#27](https://github.com/heabijay/crtcli/pull/27)) and ([#30](https://github.com/heabijay/crtcli/pull/30))

 - The string comparer can now be customized for the sorting transform in `pkg apply` ([#30](https://github.com/heabijay/crtcli/pull/30))

### Changed

 - Gave the "Terrasoft.Configuration" reference higher priority in `.csproj` sorting when using `pkg apply`-related commands

 - The `pkg apply --check` command now displays a more detailed and clearer list of files that would be applied

### Fixed

 - Fixed an issue where package installation using `app pkg install` (and related commands like `app pkg push`) could get stuck if the log fetch failed once ([#29](https://github.com/heabijay/crtcli/pull/29))


## [0.2.0](https://github.com/heabijay/crtcli/releases/tag/v0.2.0) (2025-06-01)

### Added

 - `app tunnel` command which allows to establish TCP tunnels via the Creatio instance ([#21](https://github.com/heabijay/crtcli/pull/21))

 - Stdin option to `app pkg install` using '@-' or '-' as filename ([#22](https://github.com/heabijay/crtcli/pull/22))

 - OAuth 2 authentication support to `crtcli app` commands ([#23](https://github.com/heabijay/crtcli/pull/23))

 - Flag `--force-new-session` for `crtcli app` commands just in case you need to invalidate session cache before executing ([#23](https://github.com/heabijay/crtcli/pull/23))

 - Aliases for some commands, subcommands, and options (e.g., `crtcli a p d` instead of `crtcli app pkg download`) ([#24](https://github.com/heabijay/crtcli/pull/24))

 - Flag `--force-rebuild | -f` for `crtcli app pkg compile` command


## [0.1.3](https://github.com/heabijay/crtcli/releases/tag/v0.1.3) (2025-03-25)

### Added

 - Default values for the username and password arguments in the `app` command [Supervisor:Supervisor] ([#15](https://github.com/heabijay/crtcli/pull/15))

 - `CRTCLI_INSTALL_VERSION_TAG` environment variable support in installation scripts to specify the version tag to install ([#18](https://github.com/heabijay/crtcli/pull/18))

 - `workspace.crtcli.toml` configuration file support and application aliases in `app` commands ([#19](https://github.com/heabijay/crtcli/pull/19))

### Changed

 - Replaced `--output-folder` and `--output-filename` options with the `--output` option in the `app pkg download` and `pkg pack` commands ([#14](https://github.com/heabijay/crtcli/pull/14))

 - Replaced the `--data-stdin | -D` option with the '-' value of the `--data | -d` option in the `app request` command ([#14](https://github.com/heabijay/crtcli/pull/14))

 - Refactored `app`-based commands to use the tokio async runtime, which should improve install log polling ([#16](https://github.com/heabijay/crtcli/pull/16))

 - Simplified the compile and restart options in `app pkg fs push`; also, `app pkg fs push -r` now works without the `-c` option ([#17](https://github.com/heabijay/crtcli/pull/17))

### Fixed

 - The command `app pkgs` now checks if the response is successful ([#13](https://github.com/heabijay/crtcli/pull/13))

 - The commands `app fs pull/push` and `app pkg fs pull/push` now check if the response is successful ([#17](https://github.com/heabijay/crtcli/pull/17))

 - Microsoft SQL (T-SQL) autodetection in SQL-related commands now works correctly


## [0.1.2](https://github.com/heabijay/crtcli/releases/tag/v0.1.2) (2025-01-23)

### Added

 - `--compile-package | -c` option to the `app pkg install` and `app pkg push` commands ([#9](https://github.com/heabijay/crtcli/pull/9))

 - `--check` option to the `pkg apply` command ([#10](https://github.com/heabijay/crtcli/pull/10))

### Changed

 - Simplified source and destination arguments for the `app pkg push` and `app pkg pull` commands ([#8](https://github.com/heabijay/crtcli/pull/8))

 - Improved sorting transform of data values in `Data/**/data.json` and `Data/**/Localization/data.*.json` files ([#11](https://github.com/heabijay/crtcli/pull/11))


## [0.1.1](https://github.com/heabijay/crtcli/releases/tag/v0.1.1) (2025-01-09)

### Added

 - Installation scripts for Linux, macOS, and Windows ([#1](https://github.com/heabijay/crtcli/pull/1))

 - Progress bars (spinners) for long-running commands ([#3](https://github.com/heabijay/crtcli/pull/3))

 - Support for multiple packages in the `app pkg push` command ([#4](https://github.com/heabijay/crtcli/pull/4))

 - Support for multiple packages in the `pkg pack` command ([#6](https://github.com/heabijay/crtcli/pull/6))

### Changed

 - Improved the log watcher during package installation (`app pkg install`, `app pkg push`) to create a new polling session when Creatio is running on .NET Framework (IIS) ([#5](https://github.com/heabijay/crtcli/pull/5))

 - Improved colored output messages ([#2](https://github.com/heabijay/crtcli/pull/2))

 - Changed optimization level to maximize performance

### Fixed

 - Fixed an issue where command-line options were not taking precedence over the `package.crtcli.toml` apply configuration ([#2](https://github.com/heabijay/crtcli/pull/2))


## [0.1.0](https://github.com/heabijay/crtcli/releases/tag/v0.1.0) (2025-01-03)

### Added

 - Initial Release 🎉