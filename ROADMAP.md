# Roadmap

This document tracks the work needed to turn the current macOS support from an MVP into first-class support.

当前 `codex/macos-support` 分支已经完成 macOS MVP：`zsh` / `bash` / `sh` 可以安装 shim、执行 jump、执行 alias，并且通过了 macOS 本机的 `cargo test`、`cargo clippy --all-targets -- -D warnings`、`cargo build --release` 以及临时 `HOME` 下的 shell E2E。

## Current MVP Scope

已完成：

- `--shell=zsh|bash|sh|posix` emits POSIX-compatible shell script.
- `:install zsh|bash|sh` writes a shim into `~/.zshrc`, `~/.bashrc`, or `~/.profile`.
- `:uninstall zsh|bash|sh` removes the managed shim region.
- Default config path falls back from `%USERPROFILE%` to `$HOME`.
- Windows-style absolute root paths remain valid when tests run on macOS.
- Path joining preserves the configured root style, so `C:\projects\d3` stays Windows-shaped and `/Users/me/project` stays POSIX-shaped.
- Directory completion emits the current platform's path separator.

Known MVP limitations:

- POSIX shells do not install tab completion yet.
- POSIX shells do not have the PowerShell interactive candidate UI.
- Alias command parsing still uses whitespace splitting, so shell-like quotes in `commands` are not interpreted.
- macOS support is tested locally, but CI still only runs on Windows unless the workflow is expanded.

## P0: Add macOS CI

Goal: make macOS support continuously verified.

Implementation:

- Change `.github/workflows/ci.yml` to a matrix:
  - `windows-latest`
  - `macos-latest`
- Run on both platforms:
  - `cargo test`
  - `cargo clippy --all-targets -- -D warnings`
  - `cargo build --release`
- Add a macOS-only safe E2E smoke test that uses temporary `HOME` and `J_CONFIG`:
  - build `target/release/j`
  - run `target/release/j :init zsh`
  - `eval` the generated shim in `/bin/zsh -f`
  - verify `j proj src` changes `$PWD`
  - verify `j proj -alias` runs the post-jump command

Acceptance:

- Pull requests show both Windows and macOS CI jobs.
- macOS E2E does not write to the real user profile.
- The existing Windows job remains green.

## P1: zsh Tab Completion

Goal: give macOS zsh users the same practical completion path that PowerShell users have.

Implementation:

- Extend `src/install/posix.rs` so the zsh shim installs a completion function.
- Register it with `compdef _j j`.
- Reuse existing completion subcommands:
  - `j :complete zsh <cursor> <line>` for simple candidates
  - optionally `j :complete-rich zsh <cursor> <line>` if descriptions are needed later
- Cover:
  - root names
  - child symbols
  - colon subcommands
  - aliases
  - `:install` / `:uninstall` / `:init` shell names
  - `:add` symbol-first completion with directory fallback
- Handle zsh escaping for spaces and shell metacharacters.

Acceptance:

- `j <TAB>` shows roots and subcommands.
- `j <root> <TAB>` drills into child symbols.
- `j :install <TAB>` suggests `powershell`, `cmd`, `zsh`, `bash`, `sh`.
- A temporary-home zsh E2E confirms the completion function is installed and callable.

## P2: bash Completion

Goal: support users who run bash on macOS or Linux.

Implementation:

- Generate a `_j_complete_bash` function in the bash shim.
- Use `COMP_WORDS` and `COMP_CWORD` to reconstruct the input line and cursor.
- Call `j :complete bash <cursor> <line>`.
- Register with `complete -F _j_complete_bash j`.

Acceptance:

- `j <TAB>` suggests roots and subcommands in bash.
- `j <root> <TAB>` suggests child symbols.
- Bash completion does not affect zsh or sh shims.

## P3: Shell-Like Alias Parsing

Goal: make command aliases practical for macOS commands such as `open -a "Visual Studio Code"`.

Current behavior:

- Alias command strings are split with `split_whitespace()`.
- This works for `git status`, but not for quoted arguments.

Implementation:

- Replace whitespace splitting with a shell-like tokenizer.
- Parse alias values into argv before shell emission.
- Support at least:
  - single quotes
  - double quotes
  - backslash escaping
  - empty quoted arguments
- Return `ConfigInvalid` with the alias name when parsing fails.
- Keep emission argv-based; do not `eval` arbitrary alias strings in the shell shim.

Acceptance:

- `"open -a \"Visual Studio Code\""` becomes `["open", "-a", "Visual Studio Code"]`.
- Existing simple aliases such as `"git status"` behave unchanged.
- PowerShell, cmd, and POSIX emitters continue to quote each argv element safely.

## P4: POSIX Install Options

Goal: make POSIX install behavior more flexible without surprising the user.

Implementation:

- Add optional profile path support:
  - `j :install zsh --profile ~/.zshrc`
  - `j :uninstall zsh --profile ~/.zshrc`
- Keep the default profile mapping:
  - zsh: `~/.zshrc`
  - bash: `~/.bashrc`
  - sh: `~/.profile`
- Preserve idempotent region replacement.
- Document that `j` itself must already be in a stable path, because the absolute binary path is baked into the shim.

Acceptance:

- Custom profile install/uninstall is covered by tests.
- Repeated install creates only one managed region.
- Uninstall is a no-op if the managed region is absent.

## P5: macOS Documentation

Goal: make macOS usage clear enough for users who are not reading the source.

Implementation:

- Add a dedicated macOS section to `README.md` and `README.zh-CN.md`.
- Include:
  - build command
  - recommended binary location
  - `:install zsh`
  - config path
  - common alias examples:
    - `open .`
    - `code .`
    - `git status`
  - known limitations
- Add troubleshooting:
  - `command not found: j`
  - shell profile not loaded
  - binary moved after install
  - completion not available yet

Acceptance:

- A new macOS user can build, install, configure one root, jump to it, and uninstall by following only the README.
- Known limitations are explicit.

## P6: POSIX Interactive Mode

Goal: decide whether POSIX shells should match PowerShell's no-argument interactive candidate UI.

Recommended direction:

- Do not implement shell-only TUI logic in zsh/bash functions; it is brittle.
- If interactive mode is needed, implement it in Rust as a subcommand, for example:
  - `j :interactive --shell=zsh`
- Let the shell shim only `eval` the final emitted script.

Acceptance:

- `j` with no args remains predictable.
- Any interactive mode has terminal-safe tests or is gated behind a separate command.
- The implementation does not duplicate large UI logic across zsh and bash.

