⭐ 一个清晰简洁的命令行词典，使用 Rust 实现，支持 Linux/Win/Mac

**Rust 语言实现的简洁好用的命令行词典，跨平台、高性能、持续维护更新**

## ✨ 特性

- ⚡ **极速响应，超低延迟** - Rust 的高性能保证
- 🚀 **单文件运行，多平台兼容** - 无需安装任何依赖
- 📚 **支持查单词、词组** - 本地词库（10W+），可离线使用
  > 运行时后台会自动下载数据库
- 🌐 **支持长句翻译** - 使用 `-t` 参数翻译长句
- 🎨 **灵活的配置项** - 支持修改代理、配色等
- 💾 **多级缓存策略** - 内存缓存 → 数据库缓存 → 在线查询
- 🎯 **纯英文模式** - 只显示英译/英文例句
- 📊 **状态查询** - `--status` 查看数据库和缓存状态

## 🚀 安装和编译

### 前置要求

- Rust 1.70+ (推荐使用 [rustup](https://rustup.rs/) 安装)
- Cargo (随 Rust 一起安装)

### 快速安装

#### 方式 1: 使用 cargo install（推荐）

```bash
cargo install kd-rust
```

安装后，**首次使用前需要先初始化离线词典**：

```bash
kd --update-dict
```

这将下载并设置本地词库数据库，之后即可正常使用 `kd` 命令。

#### 方式 2: 从源码编译

```bash
# 克隆项目
git clone <repository-url>
cd kd

# 编译发布版本
cargo build --release

# 安装到系统路径（可选）
sudo cp target/release/kd /usr/local/bin/kd
```

### 开发模式

```bash
# 开发模式运行
cargo run -- <query>

# 运行测试
cargo test

# 代码检查和格式化
cargo clippy
cargo fmt
```

## ⚙️ 用法

直接执行 `kd <text>` 查单词、词组（如 `kd abandon`、`kd leave me alone`）

完整用法：

```
❯ kd --help
A crystal clear command-line dictionary.

USAGE:
    kd [OPTIONS] [QUERY]...

ARGS:
    <QUERY>...    Query text

OPTIONS:
    -t, --text              Translate long query TEXT
    -n, --nocache           Don't use cached result
        --json              Output as JSON
    -T, --theme <THEME>     Choose color theme
        --update-dict       Update offline dictionary
        --generate-config   Generate config sample
        --edit-config       Edit configuration file
        --status            Show status
    -h, --help              Print help
    -V, --version           Print version
```

### 配置文件

📁 配置文件地址：Linux/MacOS 为 `~/.config/kd/config.toml`，Windows 为 `%APPDATA%\kd\config.toml`

执行 `kd --generate-config` 生成默认配置文件，执行 `kd --edit-config` 直接用编辑器打开配置文件

配置示例：

```toml
# 是否使用分页器
paging = true
# 分页器命令
# Windows: 默认使用 "more"（Windows 10+ 也支持 "less" 如果已安装）
# Linux/macOS: 默认使用 "less -RF"
# 也可以使用其他分页器，如 "bat"（需要先安装）
pager_command = "less -RF"  # Windows 会自动使用 "more" 作为默认值

# 结果中只显示英文（英译、英文例句等）
english_only = false

# 颜色主题，支持：temp/wudao
theme = "temp"

# HTTP 代理，格式：http://<IP或域名>:<端口>
http_proxy = ""

# 输出内容前自动清空终端
clear_screen = false

# 启用 emoji 字符
enable_emoji = true

# 是否开启频率提醒
freq_alert = false

# 日志配置
[logging]
  enable = true
  path = ""  # 默认：Linux/MacOS为/tmp/kd_<username>.log，Windows为%TMPDIR%/kd_<username>.log
  level = "WARN"  # 支持：DEBUG/INFO/WARN/ERROR

# 有道 API 配置（可选）
[youdao]
  api_id = ""
  api_key = ""
```

## 🏗️ 项目架构

项目采用分层架构设计，遵循领域驱动设计（DDD）和依赖倒置原则：

```
src/
├── domain/              # 领域层：核心业务模型和接口
├── infrastructure/      # 基础设施层：外部依赖实现
├── application/         # 应用层：业务逻辑
├── interfaces/          # 接口层：外部接口
├── presentation/        # 展示层：输出格式化
├── migration/           # 迁移层：数据迁移
├── state.rs             # 应用状态管理
└── main.rs              # 程序入口
```

### 核心模块

- **Domain Layer**: 定义核心业务模型、错误类型和接口（Trait）
- **Infrastructure Layer**: 实现数据库存储、HTTP 客户端、配置文件管理
- **Application Layer**: 实现查询逻辑、字典更新等业务功能
- **Interfaces Layer**: CLI 参数解析
- **Presentation Layer**: 结果格式化和主题支持

详细架构文档请参考 [ARCHITECTURE.md](./ARCHITECTURE.md)

## 🔍 查询流程

1. **CLI 参数解析** - 解析用户输入的命令和参数
2. **多级缓存查询**：
   - 内存缓存 (DashMap) ← 最快
   - 数据库缓存 (SQLite) ← 较快
   - 在线查询 (Youdao API) ← 需要网络
3. **写入缓存** - 如果找到结果，更新缓存
4. **格式化输出** - 根据主题和配置格式化显示结果

## 🎨 颜色主题

目前支持以下配色，如果不希望输出颜色，设置环境变量 `NO_COLOR=1` 即可：

- `temp` - 默认配色
- `wudao` - 复刻 Wudao Dict 的配色，鲜明易读

## 📦 依赖

主要依赖：

- `tokio` - 异步运行时
- `clap` - 命令行参数解析
- `rusqlite` / `tokio-rusqlite` - SQLite 数据库
- `reqwest` - HTTP 客户端
- `serde` / `serde_json` - 序列化/反序列化
- `toml` - 配置文件解析
- `zstd` - 数据压缩
- `dashmap` - 并发哈希表（内存缓存）
- `colored` - 终端颜色输出

完整依赖列表请参考 [Cargo.toml](./Cargo.toml)

## 🛠️ 开发

### 代码检查

```bash
# Clippy 检查
cargo clippy

# 代码格式化
cargo fmt

# 运行测试
cargo test
```

### 构建优化

发布版本已启用以下优化：

- `opt-level = 3` - 最高优化级别
- `lto = true` - 链接时优化
- `codegen-units = 1` - 单代码生成单元
- `panic = "abort"` - 减小二进制大小
- `strip = true` - 移除符号表

## ❓ 常见问题

### 数据库路径

数据库默认存储在配置目录：`~/.config/kd/kd.db` (Linux/MacOS) 或 `%APPDATA%\kd\kd.db` (Windows)

### 更新离线词典

执行 `kd --update-dict` 更新离线词典数据库

### 查看状态

执行 `kd --status` 查看数据库记录数、缓存条目数等状态信息

## 📝 许可证

请参考 [LICENSE](./LICENSE) 文件

## 🙏 致谢

本项目受以下项目启发：
- [无道词典](https://github.com/ChestnutHeng/Wudao-dict) - 提供了核心功能设计灵感
- [kd](https://github.com/Karmenzind/kd) - 提供了数据格式和实现参考
