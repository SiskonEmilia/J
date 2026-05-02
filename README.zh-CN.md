# j — Windows 确定性目录书签工具

J 是一个面向 Windows shell（PowerShell / cmd）的确定性目录书签工具。定义一次项目路径，之后用简短、可预测的命令跳转。与 zoxide、autojump 等基于历史的工具不同，J 不学习、不猜测——它只管理你明确命名的路径。单文件可执行，无依赖。

## 安装

1. 把 `j.exe` 放到任意稳定目录，例如 `C:\tools\j\j.exe`。
2. 安装 shim：

   ```powershell
   C:\tools\j\j.exe :install powershell
   ```

   ```cmd
   C:\tools\j\j.exe :install cmd
   ```

   - PowerShell：同时写入 `WindowsPowerShell`（5.1）和 `PowerShell`（7+）两个 profile，覆盖所有版本。
   - cmd：在 `%USERPROFILE%\.config\j\bin\` 生成 `j.bat`，并将该目录加入用户 PATH。

3. 新开一个 shell。PowerShell 打开新窗口；cmd 会继承新的 PATH。

> **Tip**: 安装时 `j.exe` 的绝对路径会被写入 shim 内部。如果之后移动了 `j.exe`，需要重新执行 `:install`。

也可以用 `:init <powershell|cmd>` 将 shim 脚本打印到 stdout，手动复制到自己的 profile 中。

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

  // 可复用的路径模板；模板及其子节点的 children 里都不能再 mixin 其他模板
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

合并语义：节点的 `children` 视图 = `templates` 按数组顺序依次展开 → 节点自身 children 覆盖。同名子符号：多个模板之间后者赢，节点自身赢模板；非叶节点（带 children）深合并。`:list` 输出中模板来源的符号会标注 `(template_name)`。

## 用法

```
j d3                       # 跳到 d3 根目录
j d3 d                     # → C:\projects\d3\Data（d 来自 uProject）
j d3 src pri               # → C:\projects\d3\Source\Private
j d3 d -c                  # cd 后执行 `code`
j d3 d -c --new-window     # 等效 `code --new-window`（别名后参数原样透传）
j -c --new-window          # 在当前目录执行 `code --new-window`
j                          # 显示帮助（等同 j :help）
j --help                   # 同上
j --version                # 显示版本号
j :tpl-dump d3 sharedTpl   # 将 d3 的整棵 children 封装成 template
j :tpl-apply d4 sharedTpl  # 将 sharedTpl 挂到 d4 root 上
j :tpl-apply d4 work sharedTpl  # 将 sharedTpl 挂到 d4/work 节点上
```

PowerShell 下，`j` 的 Tab 补全支持根名、子符号、别名和 `:add` 路径段：
- 首 token：补全所有根名 + 子命令
- 后续 token：逐级补全子符号
- `:add` 路径补全：先匹配子符号，无匹配时按已解析目录下的子目录补全（只列目录，行为接近 `cd`）
- 首 token 精确匹配某个根名时，展开该根的子符号供继续下钻

把 template 挂到另一个 root / 节点上：在目标节点配置 `templates` 数组即可。`:tpl-apply` 要求目标节点已在配置中存在。

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
j :list [<root> [<sym>...]]                # 树形打印（合并视图，模板来源标注 template_name）；无参 = 打印全部 roots + commands + templates
j :add <root> [<sym>...] <path>            # 新增/覆写节点；路径存原始字符串，`:check` 校验是否存在；只传 <root> <absPath> = 新增 root
j :add <root> .                            # 将当前目录记为 root
j :rm <root> [<sym>...]                    # 删除节点或 root
j :alias <name> <command>                  # 设置别名
j :alias --rm <name>                       # 删除别名
j :tpl-dump [--force] <root> [<sym>...] <tpl>  # 把 root 或目标节点的合并后 children 封装为模板（--force 覆写已有模板）
j :tpl-apply <root> [<sym>...] <tpl>       # 把模板挂到已有配置节点（目标节点必须已存在）
j :tpl-rm [--force] <tpl>                  # 删除模板（被引用时需 --force，会同时清理引用）
j :edit                                    # 用 $EDITOR / notepad 打开配置（配置不存在时自动创建默认配置）
j :check                                   # 校验所有路径存在
j :config-path                             # 打印配置文件路径
j :install   <powershell|cmd>              # 幂等写入 shim
j :uninstall <powershell|cmd>              # 反向移除
j :init      <powershell|cmd>              # 打印 shim 脚本到 stdout（手动嵌入用）
j :help | --help | -h                      # 显示帮助（末尾追加 roots 摘要）
j :version | --version                     # 显示版本号
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

### 退出码

| 码 | 含义 |
|----|------|
| 0  | 成功 |
| 1  | 内部错误 |
| 2  | 未找到（root / symbol / alias） |
| 3  | 配置错误 |
| 4  | 安装错误 |

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
