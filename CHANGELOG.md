# Changelog

## [Unreleased]

## [0.3.1] - 2026-06-14

### Fixed
- `:alias` now rejects invalid alias names before writing config, preventing
  entries like `"-cx"` that make later config loads fail.
- Non-ASCII (e.g. CJK) directory names now work on Windows. `j.exe` emits UTF-8,
  but the shims previously decoded its output using the OEM console code page,
  mangling such paths:
  - **cmd** shim switches the console to UTF-8 (`chcp 65001`) around the
    `call`/`type` step and restores the original code page afterwards.
  - **PowerShell** shim forces `[Console]::OutputEncoding = UTF-8` for the
    duration of each call (fixing Windows PowerShell 5.1) and restores it.
- Config files saved with a UTF-8 BOM (e.g. by Notepad) now load instead of
  failing to parse; the BOM is stripped on read.

## [0.3.0] - 2026-05-14

### Added
- macOS first-class support: zsh/bash/sh shim install, tab completion (zsh + bash), POSIX path handling.
- Shell-like alias tokenizer: supports single/double quotes and backslash escaping in command aliases.
- `:install/:uninstall zsh|bash|sh --profile <path>` option for custom config profiles.
- macOS CI matrix in GitHub Actions with E2E smoke test.

### Fixed
- Alias commands now tokenize properly instead of naive whitespace splitting.
- Windows CI path comparison compatibility with `canonicalize`.

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
