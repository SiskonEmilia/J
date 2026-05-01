# j — Windows 快速 cwd 跳转工具

在 PowerShell / cmd 里用 `j <root> [<sym>...] [-<alias> args...]` 跳转目录并可选执行命令；也可以用 `j -<alias> [args...]` 在当前目录直接执行别名命令。支持嵌套符号树和可复用路径模板。

## 安装

1. 把 `j.exe` 放到任意稳定目录，例如 `C:\tools\j\j.exe`。
2. 安装 shim：

   ```powershell
   C:\tools\j\j.exe :install powershell
   ```

   ```cmd
   C:\tools\j\j.exe :install cmd
   ```

3. 新开一个 shell。PowerShell 打开新窗口；cmd 会继承新的 PATH。

## 配置

配置文件位置：`%USERPROFILE%\.config\j\config.jsonc`（可用环境变量 `J_CONFIG` 覆盖）。

```jsonc
{
  // 全局扁平命令别名；调用时 -<name> 会在跳转后执行 "<cmd>" 并追加透传参数
  // 也可直接 `j -<name>`，表示在当前目录执行 "<cmd>"
  "commands": {
    "c":  "code",
    "cc": "claude",
    "g":  "git status"
  },

  // 可复用的路径模板；模板自身不能再 mixin 其他模板
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

  // 跳转根。path 必须绝对。
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

合并语义：节点的 `children` 视图 = `templates` 依次展开 → 节点自身覆盖。同名子符号后者赢；非叶节点（带 children）深合并。

## 用法

```
j d3                       # 跳到 d3 根目录
j d3 d                     # → C:\projects\d3\Data（d 来自 uProject）
j d3 src pri               # → C:\projects\d3\Source\Private
j d3 d -c                  # cd 后执行 `code`
j d3 d -c --new-window     # 等效 `code --new-window`（别名后参数原样透传）
j -c --new-window          # 在当前目录执行 `code --new-window`
j :tpl-dump d3 sharedTpl   # 将 d3 的整棵 children 封装成 template
j :tpl-apply d4 sharedTpl  # 将 sharedTpl 挂到 d4 root 上
j :tpl-apply d4 work sharedTpl  # 将 sharedTpl 挂到 d4/work 节点上
```

PowerShell 下，`j` 的 Tab 补全支持 `:add` 的路径段：
- 输入 `j :add <root> <sym>... <pathPrefix>` 时，会先解析前面的符号路径，再按该目录下的子目录补全 `<pathPrefix>`
- 如果当前位置还能匹配子符号，则优先补子符号，不抢路径补全
- 如果符号路径无法解析，或当前路径前缀没有任何匹配目录，则不返回补全结果
- 路径补全只列目录，行为更接近 `cd`
- `j` 的节点语义是目录；不要把文件路径写进 `:add`

把 template 挂到另一个 root / 节点上：在目标节点配置 `templates` 数组即可，例如：

```jsonc
{
  "templates": {
    "sharedTpl": {
      "children": {
        "notes": { "path": "docs\\notes" }
      }
    }
  },
  "roots": {
    "d3": {
      "path": "C:\\projects\\d3",
      "templates": ["sharedTpl"]
    },
    "d4": {
      "path": "C:\\projects\\d4",
      "children": {
        "work": {
          "path": "workspace",
          "templates": ["sharedTpl"]
        }
      }
    }
  }
}
```

### 子命令（冒号前缀，避免和 root 命名冲突）

```
j :list [<root> [<sym>...]]          # 树形打印（合并视图，标注来源）
j :add <root> [<sym>...] <path>      # 新增/覆写节点；路径只支持目录；只传 <root> <absPath> = 新增 root
j :add <root> .                      # 将当前目录记为 root
j :rm <root> [<sym>...]              # 删除节点或 root
j :alias <name> <command>            # 设置别名
j :alias --rm <name>                 # 删除别名
j :tpl-dump <root> [<sym>...] <tpl>  # 把 root 或目标节点的合并后 children 拷为模板
j :tpl-apply <root> [<sym>...] <tpl> # 把模板挂到已有配置节点（写入 node.templates）
j :tpl-rm <tpl>                      # 删除模板（被引用时需 --force）
j :edit                              # 用 $EDITOR / notepad 打开配置
j :check                             # 校验所有路径存在
j :config-path                       # 打印配置文件路径
j :install   <powershell|cmd>        # 幂等写入 shim
j :uninstall <powershell|cmd>        # 反向移除
j :init      <powershell|cmd>        # 打印 shim 脚本到 stdout（手动嵌入用）
j :help | :version
```

## 卸载

```powershell
C:\tools\j\j.exe :uninstall powershell
C:\tools\j\j.exe :uninstall cmd
```

## 工程说明

- Rust 2021 edition，单文件 `j.exe`，启动 <10ms。
- 核心纯函数式：argv + config → 发射目标 shell 脚本到 stdout，shim eval 之。
- 配置手写友好（JSONC，保留注释和键顺序的 CST 回写）。

### 从源码构建

```
cargo build --release
# 产物：target/release/j.exe
```

### 运行测试

```
cargo test                                   # unit + integration tests
cmd.exe /c scripts/integration.bat           # cmd shim smoke test
powershell -File scripts/integration.ps1     # PowerShell shim smoke test
```
