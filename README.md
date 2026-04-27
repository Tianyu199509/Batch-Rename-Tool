# Batch Rename Tool

> 一款极简、极速、强大的 Windows 端批量重命名工具。基于 **Tauri 2.0 (Rust + Vue 3 + TailwindCSS v4)** 构建，为你提供现代高级感的黑白灰界面和丝滑的交互体验。

## ✨ 核心特性

- **🚀 极速文件 I/O**：使用 Rust `std::fs` 底层处理，重命名操作原子且高效，瞬间完成上万文件的重命名。
- **📋 电子表格级交互**：
  - 支持 **一键提取**：可一键复制文件夹内所有原文件名或新文件名至剪贴板。
  - 支持 **全局粘贴**：支持从 Excel 中直接复制多行处理后的名称，并在工具内一键粘贴自动覆盖映射，实现极高自由度的修改。
- **🛡️ 智能后缀保护**：内置后缀名显示开关。关闭后编辑时自动隐藏后缀，避免批量修改破坏文件格式，重命名时底层智能拼合。
- **⚙️ 快速规则构建**：提供了“加前缀”、“加后缀”以及“文字查找替换”等一键操作面板。
- **📦 单文件交付**：打包产物为单个 `.exe` 免安装执行文件，携带方便。

## 📸 预览截图
*(请在此处替换为您的实际运行截图)*
![Screenshot](./docs/screenshot.png)

## 🛠️ 构建与开发指南

### 环境依赖
- **Node.js**: v18+
- **Rust**: 1.70+
- **系统要求**: Windows (自带 WebView2)

### 快速启动

1. 克隆代码仓库
```bash
git clone https://github.com/your-username/batch-rename.git
cd batch-rename
```

2. 安装前端依赖
```bash
npm install
```

3. 运行开发服务器 (会自动启动 Tauri 桌面应用)
```bash
npm run tauri dev
```

### 编译打包

如需编译出独立的免安装 `.exe` 产物，执行：
```bash
npm run tauri build
```
编译成功后，产物会输出在 `src-tauri/target/release/BatchRename.exe` 目录中。

## 📄 协议 (License)

本项目采用 [MIT License](./LICENSE) 协议开源。欢迎 Star 与 Fork！
