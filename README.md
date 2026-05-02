# j — Fast directory jump for Windows

[中文文档](README.zh-CN.md)

A single-binary directory jump tool for PowerShell and cmd. Define a tree of named symbols pointing to directories, then jump anywhere in one short command. Supports nested symbol trees, reusable path templates, and command aliases.

```
j d3              # cd to the d3 root
j d3 src pri      # cd to C:\projects\d3\Source\Private
j d3 d -c         # cd then run `code`
j -g              # run `git status` in the current directory
```

## Installation

1. Place `j.exe` in a stable directory, e.g. `C:\tools\j\j.exe`.
2. Install the shell shim:

   ```powershell
   C:\tools\j\j.exe :install powershell
   ```

   ```cmd
   C:\tools\j\j.exe :install cmd
   ```

   - **PowerShell**: writes to both `WindowsPowerShell` (5.1) and `PowerShell` (7+) profiles.
   - **cmd**: generates `j.bat` in `%USERPROFILE%\.config\j\bin\` and adds it to the user PATH.

3. Open a new shell window.

> **Tip**: The absolute path to `j.exe` is baked into the shim at install time. If you move the binary, re-run `:install`.

You can also use `:init <powershell|cmd>` to print the shim script to stdout for manual embedding.

## Configuration

Config file: `%USERPROFILE%\.config\j\config.jsonc` (override with the `J_CONFIG` env var).

```jsonc
{
  // Command aliases — `-<name>` runs "<cmd>" after jumping (or in cwd if no root given)
  "commands": {
    "c":  "code",
    "cc": "claude",
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

  // Jump roots — path must be absolute
  "roots": {
    "d3": {
      "path": "C:\\projects\\d3",
      "templates": ["uProject"],
      "children": {
        "notes": { "path": "docs\\notes" }
      }
    }
  }
}
```

**Merge semantics**: a node's effective children = templates expanded in array order, then the node's own `children` override on top. Among templates, later wins; the node's own children win over templates. Non-leaf conflicts (nodes with children) are deep-merged. `:list` annotates template-sourced symbols with the template name.

## Usage

```
j d3                       # cd to the d3 root
j d3 d                     # -> C:\projects\d3\Data (d from uProject)
j d3 src pri               # -> C:\projects\d3\Source\Private
j d3 d -c                  # cd then run `code`
j d3 d -c --new-window     # equivalent to `code --new-window` (args pass through)
j -c --new-window          # run `code --new-window` in the current directory
j                          # show help (same as j :help)
j --help                   # same as above
j --version                # print version
j :tpl-dump d3 sharedTpl   # snapshot d3's merged children as a template
j :tpl-apply d4 sharedTpl  # attach sharedTpl to the d4 root
j :tpl-apply d4 work sharedTpl  # attach sharedTpl to the d4/work node
```

### Tab completion (PowerShell)

- First token: completes root names + subcommands.
- Subsequent tokens: drills into child symbols level by level.
- `:add` path completion: matches symbols first, then falls back to subdirectories under the resolved path (directories only, similar to `cd`).
- When the first token exactly matches a root, expands its children for further drilling.

### Subcommands (colon-prefixed to avoid collisions with root names)

```
j :list [<root> [<sym>...]]                # tree view (merged, template sources annotated)
j :add <root> [<sym>...] <path>            # add/overwrite a node; root-only: <root> <absPath>
j :add <root> .                            # register cwd as a root
j :rm <root> [<sym>...]                    # remove a node or root
j :alias <name> <command>                  # set a command alias
j :alias --rm <name>                       # remove an alias
j :tpl-dump [--force] <root> [<sym>...] <tpl>  # export merged children as template
j :tpl-apply <root> [<sym>...] <tpl>       # attach template to an existing node
j :tpl-rm [--force] <tpl>                  # delete template (--force if referenced)
j :edit                                    # open config in $EDITOR / notepad
j :check                                   # validate all paths exist
j :config-path                             # print config file path
j :install   <powershell|cmd>              # idempotent shim install
j :uninstall <powershell|cmd>              # remove shim
j :init      <powershell|cmd>              # print shim script to stdout
j :help | --help | -h                      # show help
j :version | --version                     # print version
```

## Uninstallation

```powershell
C:\tools\j\j.exe :uninstall powershell
C:\tools\j\j.exe :uninstall cmd
```

## Design

- Rust 2021 edition, single binary `j.exe`, startup <10 ms.
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

### Building from source

```
cargo build --release
# output: target/release/j.exe
```

### Running tests

```
cargo test                                   # unit + integration tests
cmd.exe /c scripts/integration.bat           # cmd shim smoke test
powershell -File scripts/integration.ps1     # PowerShell shim smoke test
```

## License

This project is licensed under the [GNU General Public License v3.0](LICENSE).
