# keentools-cloud CLI

**中文** | [English](README.md)

[![CI](https://github.com/loonghao/keentools_cloud_cli/actions/workflows/ci.yml/badge.svg)](https://github.com/loonghao/keentools_cloud_cli/actions/workflows/ci.yml)
[![Release](https://img.shields.io/github/v/release/loonghao/keentools_cloud_cli?label=release)](https://github.com/loonghao/keentools_cloud_cli/releases/latest)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-stable-orange.svg)](https://www.rust-lang.org)
[![Platform](https://img.shields.io/badge/platform-Linux%20%7C%20macOS%20%7C%20Windows-lightgrey)](#安装)
[![Downloads](https://img.shields.io/github/downloads/loonghao/keentools_cloud_cli/total)](https://github.com/loonghao/keentools_cloud_cli/releases)

> **[KeenTools Cloud](https://keentools.io) 3D 头部重建 API 的非官方 CLI 工具。**
> 本项目与 KeenTools 无任何关联或官方背书。

使用 2–15 张普通照片，直接在终端或自动化工作流中生成逼真的 3D 头部模型。基于 Rust 构建，遵循 [Agent DX 最佳实践](https://justin.poehnelt.com/posts/rewrite-your-cli-for-ai-agents/)，可被 Claude、OpenClaw 等 AI 智能体可靠地调用。

## 特性

- **一条命令完成全流程** — `run` 命令涵盖初始化 → 上传 → 重建 → 下载
- **Agent 优先设计** — JSON 输出、`--dry-run`、`schema` 内省、输入安全校验
- **标准流水线与临时流水线** — 云端存储或零数据留存（照片不保存到服务器）
- **灵活认证** — 环境变量、`--token` 参数或配置文件
- **终端进度条**；管道/重定向时自动切换为 NDJSON 流式输出

## 安装

### 方式一 — 一键安装（Linux / macOS）

```bash
curl -fsSL https://raw.githubusercontent.com/loonghao/keentools_cloud_cli/main/install.sh | bash
```

安装指定版本：

```bash
curl -fsSL https://raw.githubusercontent.com/loonghao/keentools_cloud_cli/main/install.sh | bash -s -- v0.1.1
```

### 方式二 — PowerShell（Windows）

```powershell
irm https://raw.githubusercontent.com/loonghao/keentools_cloud_cli/main/install.ps1 | iex
```

### 方式三 — 下载预编译二进制

从 [Releases 页面](https://github.com/loonghao/keentools_cloud_cli/releases/latest) 下载对应平台的二进制文件，放到 `PATH` 中即可。

| 平台 | 压缩包 |
|------|--------|
| Linux x86_64 | `keentools-cloud-*-x86_64-unknown-linux-gnu.tar.gz` |
| Linux ARM64 | `keentools-cloud-*-aarch64-unknown-linux-gnu.tar.gz` |
| macOS x86_64 | `keentools-cloud-*-x86_64-apple-darwin.tar.gz` |
| macOS ARM64（M1+） | `keentools-cloud-*-aarch64-apple-darwin.tar.gz` |
| Windows x64 | `keentools-cloud-*-x86_64-pc-windows-msvc.zip` |

### 方式四 — 从源码构建

需要 [Rust](https://rustup.rs/) stable 工具链。

```bash
git clone https://github.com/loonghao/keentools_cloud_cli
cd keentools_cloud_cli
cargo build --release
# 二进制文件：./target/release/keentools-cloud
```

或直接从 Git 安装：

```bash
cargo install --git https://github.com/loonghao/keentools_cloud_cli
```

### 自动更新

```bash
keentools-cloud self-update
```

## 快速开始

```bash
# 设置 API Token 和接入地址
export KEENTOOLS_API_TOKEN=your_token_here
export KEENTOOLS_API_URL=https://your-api-endpoint.example.com

# 一条命令完成完整流程
keentools-cloud run photo1.jpg photo2.jpg photo3.jpg \
  --output-path head.glb \
  --blendshapes arkit,nose \
  --texture jpg
```

## 命令概览

| 命令 | 说明 |
|------|------|
| `run` | 完整流程快捷命令（推荐） |
| `init` | 初始化会话，获取上传 URL |
| `upload` | 将照片上传到预签名 S3 URL |
| `process` | 启动重建 |
| `status` | 查询状态（用 `--poll` 等待完成） |
| `download` | 下载 3D 模型（用 `--poll`） |
| `info` | 获取相机矩阵等重建元数据 |
| `ephemeral` | 零数据留存流水线 |
| `schema` | 输出完整 CLI 模式信息（JSON） |
| `auth` | 管理存储的 API Token |
| `self-update` | 更新 CLI 到最新版本 |

## 分步流程

```bash
# 1. 初始化
INIT=$(keentools-cloud init --count 3 --output json)
AVATAR_ID=$(echo "$INIT" | jq -r .avatar_id)

# 2. 上传照片
keentools-cloud upload \
  --avatar-id "$AVATAR_ID" \
  --urls "$(echo "$INIT" | jq -r '.upload_urls | join(",")')" \
  photo1.jpg photo2.jpg photo3.jpg

# 3. 启动重建
keentools-cloud process \
  --avatar-id "$AVATAR_ID" \
  --focal-length-type estimate-per-image

# 4. 等待完成
keentools-cloud status --avatar-id "$AVATAR_ID" --poll

# 5. 下载模型
keentools-cloud download \
  --avatar-id "$AVATAR_ID" \
  --output-path head.glb \
  --format glb \
  --blendshapes arkit,nose \
  --poll
```

## 临时流水线（零数据留存）

照片在内存中处理，结果直接推送到您提供的预签名 URL，KeenTools 服务器不保留任何数据。

```bash
keentools-cloud ephemeral \
  --image-url https://your-storage.com/photo1.jpg \
  --image-url https://your-storage.com/photo2.jpg \
  --result-url glb:https://your-storage.com/result.glb?<presigned-put> \
  --focal-length-type estimate-per-image \
  --callback-url https://your-server.com/webhook
```

> **注意：** 临时模式下，处理完成后 `download` 和 `info` 端点不可用。结果只存在于您提供的 URL 中。

## 示例代码

[`examples/`](examples/) 目录包含开箱即用的示例：

| 文件 | 说明 |
|------|------|
| [`examples/cli-quickstart.md`](examples/cli-quickstart.md) | 所有 CLI 子命令的完整使用指南，含可直接复制的示例 |
| [`examples/pipeline.py`](examples/pipeline.py) | 直接调用 REST API 的完整 Python 流水线（需 `pip install requests`） |
| [`examples/pipeline.sh`](examples/pipeline.sh) | 完整 Bash 流水线（需 `curl` + `jq`） |
| [`examples/ipc-qt-demo.py`](examples/ipc-qt-demo.py) | PySide6/PyQt6 桌面应用，通过 `--ipc` NDJSON 流实时展示进度 |
| [`examples/web-bridge.py`](examples/web-bridge.py) | Flask + Server-Sent Events 浏览器前端，由 `--ipc` 模式驱动（需 `pip install flask`） |
| [`examples/actionforge-guide.md`](examples/actionforge-guide.md) | 与 [Actionforge](https://docs.actionforge.dev/agentic-coding/) 的智能代理集成指南，含 MCP 配置和 `.act` 图谱示例 |

```bash
export KEENTOOLS_API_URL=https://your-api-endpoint.example.com
export KEENTOOLS_API_TOKEN=your_token_here

# Python REST 流水线
python examples/pipeline.py photo1.jpg photo2.jpg photo3.jpg

# Bash 流水线
bash examples/pipeline.sh photo1.jpg photo2.jpg photo3.jpg

# Qt 桌面应用（实时进度）
pip install PySide6
python examples/ipc-qt-demo.py

# 浏览器前端
pip install flask
python examples/web-bridge.py   # 打开 http://localhost:5000
```

## 输出格式

- **终端 (TTY)**：彩色人类可读输出
- **管道/重定向 (非 TTY)**：自动切换为 JSON
- **强制 JSON**：`--output json`

```bash
# 在自动化流水线中使用
keentools-cloud status --avatar-id abc123 --output json
# {"status":"completed"}
```

## 认证方式

优先级顺序：
1. `--token <TOKEN>` 参数
2. `KEENTOOLS_API_TOKEN` 环境变量
3. `~/.config/keentools-cloud/config.toml`

API 接入地址通过 `KEENTOOLS_API_URL` 环境变量或 `--api-url` 参数指定（必填）。

```bash
# 永久存储 Token
keentools-cloud auth login <token>

# 查看当前认证状态
keentools-cloud auth status

# 删除存储的 Token
keentools-cloud auth logout
```

## Agent 集成

本 CLI 专为在 AI Agent 工作流中可靠运行而设计：

```bash
# 运行时查看所有命令和参数
keentools-cloud schema

# 查看特定命令的 schema
keentools-cloud schema run

# 变更操作前先 dry-run
keentools-cloud init --count 3 --dry-run
keentools-cloud process --avatar-id abc --focal-length-type estimate-per-image --dry-run
```

完整的 Agent 使用约定请参见 [SKILL.md](SKILL.md)。

## 输出格式说明

### GLB（推荐）
单个二进制文件（约 40 MB），内嵌 JPEG 纹理、线框边缘和变形目标。使用 `--format glb`。

### OBJ
始终以 **ZIP 压缩包**形式返回，包含 `.obj`、`.mtl` 和纹理文件。使用 `--format obj`。

## 混合变形（Blendshapes，仅 GLB）

| 组 | 说明 |
|----|------|
| `arkit` | 51 个 ARKit 兼容变形目标 |
| `expression` | 编号表情变形（需在 `process` 时使用 `--expressions`） |
| `nose` | 鼻形控制 |

```bash
--blendshapes arkit,nose    # 逗号分隔，禁止重复传参
```

## 退出码

| 码 | 含义 |
|----|------|
| 0 | 成功 |
| 1 | API 或运行时错误 |
| 2 | 输入校验错误 |
| 3 | 认证错误 |

## 免责声明

本工具为**非官方**项目，与 KeenTools 无任何关联或背书。官方支持请访问 [keentools.io](https://keentools.io)。
