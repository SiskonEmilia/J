# Changelog

## [0.2.0] - 2025-05-01

### Added
- Reusable path templates (`templates` in config, `:tpl-dump` / `:tpl-apply` / `:tpl-rm` subcommands).
- `:list` annotates template-sourced symbols with their template name.
- Tab completion: exact-root expansion (typing a root name and pressing Tab drills into its children).

### Fixed
- PowerShell 5.1 tab completion compatibility.
- Replaced Unicode arrow with ASCII `->` in `:list` output to avoid garbled text on Windows consoles.

## [0.1.0] - 2025-04-28

Initial release.

- Nested symbol tree for directory jumping in PowerShell and cmd.
- Command aliases (`commands` in config) with argument pass-through.
- Subcommands: `:add`, `:rm`, `:alias`, `:list`, `:check`, `:edit`, `:config-path`, `:install`, `:uninstall`, `:init`, `:help`, `:version`.
- PowerShell tab completion (root names, child symbols, aliases, `:add` path segments).
- JSONC config with comment-preserving CST rewrite.
- Idempotent shim install for PowerShell (5.1 + 7+) and cmd.
