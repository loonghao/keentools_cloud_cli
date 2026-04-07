# keentools-cloud CLI

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

```bash
cargo install --git https://github.com/loonghao/keentools_cloud_cli
```

或从源码构建：

```bash
git clone https://github.com/loonghao/keentools_cloud_cli
cd keentools_cloud_cli
cargo build --release
# 二进制文件：./target/release/keentools-cloud
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
