# j — Deterministic directory bookmarks

[中文文档](README.zh-CN.md)

J is a deterministic directory bookmark tool for Windows and macOS shells (PowerShell, cmd, zsh, bash, and sh). Define named project paths once, then jump to them with short, predictable commands. Unlike history-based tools such as zoxide or autojump, J does not learn or guess — it is for paths you already know and want to name. Ships as a single binary with no dependencies.

```
j proj             # cd to the proj root
j proj src lib     # cd to deeper path
j proj -c          # cd then run `code`
j -g               # run `git status` in the current directory
```

## Installation

### Download a release

Prebuilt binaries for **Windows** and **macOS** are available on the [Releases page](https://github.com/SiskonEmilia/J/releases):

- `j-{version}-x86_64-pc-windows-msvc.exe`
- `j-{version}-aarch64-apple-darwin`

Place the binary in a stable directory (e.g. `C:\tools\j\j.exe` on Windows, `/usr/local/bin/j` on macOS), then install the shim:

```powershell
# Windows (PowerShell)
C:\tools\j\j.exe :install powershell
```

```cmd
rem Windows (cmd)
C:\tools\j\j.exe :install cmd
```

```sh
# macOS
/usr/local/bin/j :install zsh       # ~/.zshrc
/usr/local/bin/j :install bash      # ~/.bashrc
```

- **PowerShell**: writes to both `WindowsPowerShell` (5.1) and `PowerShell` (7+) profiles.
- **cmd**: generates `j.bat` in `%USERPROFILE%\.config\j\bin\` and adds it to the user PATH.
- **zsh/bash**: writes a shim function + tab completion into the profile. `:install zsh` also registers `compdef _j j`; `:install bash` registers `complete -F _j_complete_bash j`.
- **sh**: writes a shim into `~/.profile` (no completion).

Open a new shell window.

> **Tip**: The absolute path to the binary is baked into the shim at install time. If you move the binary, re-run `:install`.

You can also use `:init <shell>` to print the shim script to stdout for manual embedding.

### Build from source

```sh
cargo build --release
# output: target/release/j  (macOS)  or  target/release/j.exe  (Windows)
```

## Configuration

Config file: `%USERPROFILE%\.config\j\config.jsonc` on Windows, `~/.config/j/config.jsonc` on macOS (override with the `J_CONFIG` env var).

```jsonc
{
  // Command aliases — `-<name>` runs "<cmd>" after jumping (or in cwd if no root given)
  "commands": {
    "c":  "code",
    "o":  "open .",
    "g":  "git status"
  },

  // Reusable path templates (cannot nest templates inside templates)
  "templates": {
    "uProject": {
      "children": {
        "d":   { "path": "Data" },
        "sd":  { "path": "Shared/Data" },
        "src": {
          "path": "Source",
          "children": {
            "pri": { "path": "Private" },
            "pub": { "path": "Public" }
          }
        }
      }
    }
  },

  // Jump roots — path must be absolute (POSIX or Windows style)
  "roots": {
    "proj": {
      "path": "/Users/me/projects/myproject",
      // or on Windows: "path": "C:\\projects\\myproject",
      "templates": ["uProject"],
      "children": {
        "notes": { "path": "docs/notes" }
      }
    }
  }
}
```

**Merge semantics**: a node's effective children = templates expanded in array order, then the node's own `children` override on top. Among templates, later wins; the node's own children win over templates. Non-leaf conflicts (nodes with children) are deep-merged. `:list` annotates template-sourced symbols with the template name.

## Usage

```
j proj                      # cd to the proj root
j proj d                    # -> proj/Data (d from uProject template)
j proj src pri              # -> proj/Source/Private
j proj -c                   # cd then run `code`
j proj -c --new-window      # equivalent to `code --new-window` (args pass through)
j -c --new-window           # run `code --new-window` in the current directory
j                           # show help (same as j :help)
j --help                    # same as above
j --version                 # print version
j :tpl-dump proj sharedTpl  # snapshot proj's merged children as a template
j :tpl-apply proj2 sharedTpl # attach sharedTpl to another root
```

### Tab completion

**PowerShell**: completes root names, subcommands, child symbols, aliases, and `:add` path segments. Exact root match expands its children for further drilling. `:add` completion matches symbols first, then falls back to subdirectories under the resolved path.

**zsh / bash**: installed automatically with `:install zsh` / `:install bash`. Covers root names, child symbols, colon subcommands, aliases, shell names for `:install`/`:uninstall`/`:init`, and `:add` directory fallback.

### Subcommands (colon-prefixed to avoid collisions with root names)

```
j :list [<root> [<sym>...]]                     # tree view (merged, template sources annotated)
j :add <root> [<sym>...] <path>                 # add/overwrite a node; root-only: <root> <absPath>
j :add <root> .                                 # register cwd as a root
j :rm <root> [<sym>...]                         # remove a node or root
j :alias <name> <command>                       # set a command alias
j :alias --rm <name>                            # remove an alias
j :tpl-dump [--force] <root> [<sym>...] <tpl>   # export merged children as template
j :tpl-apply <root> [<sym>...] <tpl>            # attach template to an existing node
j :tpl-rm [--force] <tpl>                       # delete template (--force if referenced)
j :edit                                         # open config in $EDITOR
j :check                                        # validate all paths exist
j :config-path                                  # print config file path
j :install   <shell> [--profile <path>]         # idempotent shim install
j :uninstall <shell> [--profile <path>]         # remove shim
j :init      <shell>                            # print shim script to stdout
j :help | --help | -h                           # show help
j :version | --version                          # print version
```

`--profile <path>` is available for POSIX shells (zsh/bash/sh) to install into a custom profile file:

```sh
j :install zsh --profile ~/.my_custom_zshrc
```

## Alias quoting

Alias command strings support shell-like quoting (single quotes, double quotes, backslash escaping):

```jsonc
"commands": {
  "vsc": "open -a \"Visual Studio Code\"",
  "echo": "echo 'hello world'",
  "path": "ls /Users/me/My\\ Documents"
}
```

## Uninstallation

```powershell
# Windows
C:\tools\j\j.exe :uninstall powershell
C:\tools\j\j.exe :uninstall cmd
```

```sh
# macOS
/usr/local/bin/j :uninstall zsh
```

## Design

- Rust 2021 edition, single binary, startup <10 ms.
- Pure-functional core: argv + config -> shell script emitted to stdout, evaluated by the shim.
- Hand-edit friendly config (JSONC with comment-preserving, key-order-preserving CST rewrite).

### Exit codes

| Code | Meaning |
|------|---------|
| 0    | Success |
| 1    | Internal error |
| 2    | Not found (root / symbol / alias) |
| 3    | Config error |
| 4    | Install error |

### Running tests

```sh
cargo test                                   # unit + integration tests
cmd.exe /c scripts/integration.bat           # cmd shim smoke test (Windows)
powershell -File scripts/integration.ps1     # PowerShell shim smoke test (Windows)
bash scripts/e2e-macos.sh                    # POSIX E2E smoke test (macOS)
```

## License

This project is licensed under the [GNU General Public License v3.0](LICENSE).
