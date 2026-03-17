<div align="center">

# 🐱 Moocha

**An AI-powered desktop pet · AI 驱动的桌面宠物**

*Inspired by the gentle soul of the Siberian Forest Cat*

[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)
[![Tauri](https://img.shields.io/badge/Tauri-v2-24C8D8?logo=tauri)](https://tauri.app)
[![Rust](https://img.shields.io/badge/Rust-1.77+-orange?logo=rust)](https://www.rust-lang.org)
[![React](https://img.shields.io/badge/React-19-61DAFB?logo=react)](https://react.dev)
[![Build](https://img.shields.io/badge/build-passing-brightgreen)](#开发指南--development-guide)
[![PRs Welcome](https://img.shields.io/badge/PRs-welcome-brightgreen.svg)](CONTRIBUTING.md)

</div>

---

## 简介 · Introduction

**Moocha** 是一款开源的 AI 桌面宠物应用，以西伯利亚森林猫为原型。它常驻于桌面一角，拥有独立的情绪与行为系统，并通过可配置的 AI 后端（本地模型或云端 API）与你自然对话。

**Moocha** is an open-source AI desktop companion modeled after the Siberian Forest Cat. It lives in a corner of your screen, expresses moods through animations, and converses naturally via a configurable AI backend — whether local or cloud-based.

> "不打扰，但随时都在。"  
> *"Always present, never intrusive."*

---

## 截图 · Screenshots

> 🚧 Coming soon — screenshots will be added after the first UI release.

<!--
<div align="center">
  <img src="docs/screenshots/idle.png" alt="Moocha Idle" width="200"/>
  <img src="docs/screenshots/happy.png" alt="Moocha Happy" width="200"/>
  <img src="docs/screenshots/thinking.png" alt="Moocha Thinking" width="200"/>
</div>
-->

---

## 特性 · Features

| 特性 | Feature |
|------|---------|
| 🦀 **Rust 驱动后端** | Rust-powered backend for safety & performance |
| 🪟 **透明无边框窗口** | Frameless & transparent window, always on top |
| 🤖 **多 AI 后端支持** | OpenAI / Claude / Ollama (local) — plug-and-play |
| 🔒 **无硬编码密钥** | Zero hardcoded API keys, env-var driven config |
| 💬 **自然语言对话** | Chat naturally via the pet interaction UI |
| 😸 **情绪状态系统** | Mood system: idle / happy / thinking / sleeping |
| 📦 **跨平台发布** | Cross-platform: Windows, macOS, Linux |
| 🔧 **可扩展架构** | Extensible `AIProvider` trait for custom backends |

---

## 技术栈 · Tech Stack

```
Frontend   │ React 19 + TypeScript + Vite
Backend    │ Rust (Tauri v2)
AI Layer   │ reqwest + async-trait (OpenAI-compatible API)
Logging    │ tracing + tracing-subscriber
Config     │ serde_json (JSON config file)
Packaging  │ Tauri Bundler (NSIS / DMG / AppImage)
```

---

## 安装指南 · Installation

### 预编译版本 · Pre-built Binaries

前往 [Releases](https://github.com/your-username/moocha/releases) 页面下载对应平台的安装包。

Go to the [Releases](https://github.com/your-username/moocha/releases) page and download the installer for your platform.

| 平台 Platform | 安装包格式 Installer |
|--------------|---------------------|
| Windows 10/11 | `.msi` / `.exe` (NSIS) |
| macOS 12+     | `.dmg` (Universal) |
| Linux         | `.AppImage` / `.deb` |

---

## 开发指南 · Development Guide

### 环境要求 · Prerequisites

| 工具 Tool | 版本 Version | 说明 Notes |
|-----------|-------------|------------|
| [Rust](https://rustup.rs) | 1.77+ | `rustup update stable` |
| [Node.js](https://nodejs.org) | 18+ | LTS recommended |
| [pnpm](https://pnpm.io) | 8+ | `npm i -g pnpm` |
| WebView2 | — | Windows: pre-installed on Win11; [download](https://developer.microsoft.com/en-us/microsoft-edge/webview2/) for Win10 |

**Linux 额外依赖 · Linux extra dependencies:**

```bash
# Ubuntu / Debian
sudo apt install libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev patchelf

# Fedora
sudo dnf install webkit2gtk4.1-devel libappindicator-gtk3-devel librsvg2-devel
```

### 克隆与启动 · Clone & Run

```bash
# 克隆仓库 Clone the repo
git clone https://github.com/your-username/moocha.git
cd moocha

# 安装前端依赖 Install frontend dependencies
pnpm install

# 启动开发模式（热重载）Start dev mode with hot-reload
pnpm tauri dev
```

### 构建生产包 · Build for Production

```bash
pnpm tauri build
# 输出位于 src-tauri/target/release/bundle/
# Output located in src-tauri/target/release/bundle/
```

### 验证 Rust 代码 · Verify Rust Code

```bash
cd src-tauri
cargo check     # 快速类型检查 Quick type-check
cargo clippy    # Lint 检查 Lint check
cargo test      # 运行测试 Run tests
```

---

## 配置说明 · Configuration

### 环境变量 · Environment Variables

在项目根目录创建 `.env` 文件（已被 `.gitignore` 保护，**永远不要提交此文件**）：

Create a `.env` file in the project root (protected by `.gitignore` — **never commit this file**):

```env
# AI 服务 API Key（必填）
# AI service API key (required)
MOOCHA_API_KEY=sk-xxxxxxxxxxxxxxxxxxxxxxxx

# 日志级别（可选，默认 info）
# Log level (optional, default: info)
RUST_LOG=moocha=debug
```

### 应用配置文件 · App Config File

应用首次运行后会在系统数据目录生成 `config.json`，也可提前创建：

After the first run, `config.json` is generated in the system data directory. You can also create it manually:

**Windows:** `%APPDATA%\moocha\config.json`  
**macOS:** `~/Library/Application Support/moocha/config.json`  
**Linux:** `~/.local/share/moocha/config.json`

```json
{
  "base_url": "https://api.openai.com/v1",
  "model_name": "gpt-4o-mini",
  "provider_type": "open_ai_compatible"
}
```

### 使用本地模型 · Using a Local Model (Ollama)

```bash
# 安装 Ollama Install Ollama
# https://ollama.com

# 拉取模型 Pull a model
ollama pull llama3.2

# 在 config.json 中设置 Set in config.json
```

```json
{
  "base_url": "http://localhost:11434/v1",
  "model_name": "llama3.2",
  "provider_type": "open_ai_compatible"
}
```

本地模型无需设置 `MOOCHA_API_KEY`，或将其设为任意字符串。

No `MOOCHA_API_KEY` needed for local models — set it to any placeholder string.

---

## 项目结构 · Project Structure

```
moocha/
├── src/                    # React 前端 Frontend
│   ├── App.tsx             # 根组件 Root component
│   ├── App.css             # 透明窗口样式 Transparent styles
│   └── index.css           # 全局透明基础样式 Global base styles
│
├── src-tauri/              # Rust 后端 Backend
│   ├── src/
│   │   ├── ai/
│   │   │   ├── mod.rs      # AI 模块导出 Module exports
│   │   │   └── provider.rs # AIProvider trait 定义
│   │   ├── config.rs       # AppConfig 配置结构体
│   │   ├── state.rs        # 全局共享状态 Global state
│   │   ├── lib.rs          # Tauri 入口 & Commands
│   │   └── main.rs         # 程序入口 Entry point
│   ├── Cargo.toml
│   └── tauri.conf.json     # 窗口 & 打包配置 Window & bundle config
│
├── .env                    # 🔒 本地密钥（不提交）Local secrets
├── .env.example            # 配置模板 Config template
└── README.md
```

---

## 贡献指南 · Contributing

欢迎任何形式的贡献！All contributions are welcome!

### 提交 Issue

- 🐛 **Bug 报告**：请附上复现步骤、操作系统版本和日志（`RUST_LOG=debug pnpm tauri dev`）
- 💡 **功能建议**：描述使用场景和期望行为

### 提交 Pull Request

```bash
# 1. Fork 本仓库并克隆 Fork & clone
git clone https://github.com/your-username/moocha.git

# 2. 创建功能分支 Create a feature branch
git checkout -b feat/your-feature-name

# 3. 提交前检查 Pre-commit checks
cd src-tauri && cargo clippy && cargo test

# 4. 推送并创建 PR Push and open PR
git push origin feat/your-feature-name
```

### 代码规范 · Code Style

- **Rust**: 遵循 `rustfmt` 默认格式，使用 `clippy` 消除 warning
- **TypeScript**: 遵循项目 ESLint 配置
- **提交信息**: 使用 [Conventional Commits](https://www.conventionalcommits.org/) 格式
  ```
  feat: add sleeping animation for idle timeout
  fix: prevent window from hiding behind taskbar
  docs: update configuration guide
  ```

---

## 路线图 · Roadmap

- [ ] 🎨 西伯利亚森林猫动画精灵 Animated sprite (Siberian Forest Cat)
- [ ] 💬 对话气泡 UI Chat bubble interface
- [ ] 🧠 对话历史记忆 Conversation history & memory
- [ ] 🖱️ 点击互动 Click interactions & petting
- [ ] 🔔 系统托盘集成 System tray integration
- [ ] 🌙 自动休眠模式 Auto-sleep on idle
- [ ] 🧩 插件系统 Plugin system for custom behaviors
- [ ] 🌐 多语言支持 i18n support

---

## License

本项目基于 **Apache License 2.0** 开源。

This project is licensed under the **Apache License 2.0**.

See the [LICENSE](LICENSE) file for full details.

---

<div align="center">

Made with ❤️ and 🦀 Rust

*如果 Moocha 陪伴了你的工作，欢迎点一颗 ⭐*  
*If Moocha keeps you company while you work, consider leaving a ⭐*

</div>
