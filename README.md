
# DLsite Rust Renamer

基于 Rust 编写的 DLsite 作品重命名工具，支持自动抓取作品信息并根据预设规则批量重命名本地文件夹，适用于 Windows 平台。

> 🚧 项目仍在开发中，部分功能尚未完成，欢迎贡献！

## ✨ 功能特性

- 🔍 **自动识别 RJ 号**：从文件夹名称或路径中识别出 RJ 号（如 `RJ123456`）。
- 🌐 **联网抓取元数据**：通过网络请求自动获取作品的标题、创作者、发售日期、类型标签等信息。
- 🏷️ **重命名规则灵活可配**：支持使用模板规则自动重命名文件夹（如：`[创作者] 标题 (RJ123456)`）。
- 🖼️ **下载封面图**：可选下载作品封面并保存至本地。
- 🧪 **图形界面（Egui）支持**：提供基础图形界面供操作与查看日志。
- 📁 **批量重命名支持**：支持扫描指定目录下多个作品文件夹并批量处理。

## 🛠️ 构建方式

本项目使用 Rust 编写，构建流程如下：

### 1. 安装 Rust 环境

请确保你已经安装了 Rust 工具链，推荐使用 [rustup](https://www.rust-lang.org/zh-CN/tools/install)。

```bash
rustup update
````

### 2. 克隆项目

```bash
git clone https://github.com/mdzzdsbhz/DLsite_rust_renamer.git
cd DLsite_rust_renamer
```

### 3. 构建发布版（release）

```bash
cargo build --release
```

构建完成后，二进制文件将位于：

```
target/release/DLsite_rust_renamer.exe
```

### 4. 运行程序

```bash
cargo run --release
```

或直接运行编译后的可执行文件：

```bash
./target/release/DLsite_rust_renamer.exe
```

## 🧩 使用说明（待完善）

> 💡 当前默认扫描当前目录下的所有文件夹，请确保每个作品目录中包含可识别的 RJ 号。

后续将补充详细使用教程和截图。懒惰了，看着用吧，勉强能用

---

## 🚧 未来计划 / 未实现功能

* [ ] 重要！没有实现图片替换为windows文件夹封面，仅实现了下载cover到本地，非常拉！proxy功能好像也有点问题，需要系统代理
* [ ] 支持 RJ 号以外的作品编号（如 BJ, VJ 等）
* [ ] 支持更多重命名模板配置（用户可自定义）
* [ ] 添加设置面板保存配置到本地
* [ ] 日志导出与错误重试机制
* [ ] 下载音频样本 / 文本介绍等附加内容
* [ ] CLI 模式增强（非 GUI 使用场景）
* [ ] 跨平台支持（macOS / Linux）

---

## 📦 项目结构简要说明

| 文件 / 模块           | 说明               |
| ----------------- | ---------------- |
| `src/main.rs`     | 应用入口，初始化 GUI     |
| `src/renamer.rs`  | 重命名核心逻辑实现        |
| `src/dlsite/*.rs` | 与 DLsite 元数据解析相关 |
| `src/config.rs`   | 配置加载与模板格式化       |
| `src/gui.rs`      | Egui 图形界面实现      |
| `fonts/`         | 本地资源（字体等）        |

---

## 📄 License

本项目基于 MIT 协议开源，欢迎自由使用与修改。

