# Toolkit 应用 - 设计文档

## 应用定位

一款个人工具箱桌面应用，使用侧边栏导航容纳多个独立的实用工具模块。目前规划的首个模块：Claude Code 配置管理。

## 技术栈

基于现有的模板项目：

- Tauri v2 (Rust 后端)
- React + TypeScript
- shadcn/ui + Tailwind CSS v4
- Jotai (状态管理)
- TanStack Query (数据获取)
- SQLite (本地存储)

## 整体布局

- **左侧侧边栏**：固定导航，列出所有工具模块（图标 + 名称），点击切换
- **右侧内容区**：当前模块的主界面
- **模块化设计**：每个模块都是独立的路由 + 页面组件，共享基础 UI 和存储层

---

## 模块：Claude Code 配置管理

### 核心概念

管理多个 Claude Code 启动配置文件（Profile）。每个配置文件本质上是一组命名的环境变量集合，这些变量将生成为一个 Shell 别名（alias）并写入 `~/.zshrc` 文件中。

### 数据模型

```
Profile
├── id: string (自动生成的 UUID)
├── name: string (显示名称，例如 "Leo")
├── alias: string (终端命令名称，例如 "ccleo")
├── envVars: Array<{ key: string, value: string }>
│   例如：
│   ├── ANTHROPIC_BASE_URL = "https://claude.leocoder.cn"
│   └── ANTHROPIC_AUTH_TOKEN = "sk-0dd3fd..."
├── createdAt: timestamp
└── updatedAt: timestamp
```

### 生成的别名格式

```bash
alias ccleo="ANTHROPIC_BASE_URL=https://claude.leocoder.cn ANTHROPIC_AUTH_TOKEN=sk-0dd3fd... claude"
```

### UI 交互

- **配置列表**：表格或卡片布局，显示所有配置（名称、别名、环境变量数量）
- **添加/编辑**：模态框表单，包含名称、别名输入框，以及用于环境变量的动态键值对行
- **同步到 .zshrc**：点击按钮将所有配置别名写入 `~/.zshrc`（在标记区域内）
- **API Key 脱敏**：列表默认将 Key 类值显示为 `sk-****`

### .zshrc 写入策略

**标记区域方案**：使用注释标记在 `.zshrc` 中定义专属区域。仅该区域内的内容由应用管理。

```bash
# === CLAUDE_CODE_ALIAS_START (DO NOT EDIT MANUALLY) ===
alias ccleo="ANTHROPIC_BASE_URL=https://claude.leocoder.cn ANTHROPIC_AUTH_TOKEN=sk-0dd3fd... claude"
alias ccother="ANTHROPIC_BASE_URL=https://other.api.com ANTHROPIC_AUTH_TOKEN=sk-abc123... claude"
# === CLAUDE_CODE_ALIAS_END ===
```

**安全措施**：

- 每次写入前将 `~/.zshrc` 备份为 `~/.zshrc.bak`
- 仅替换标记区域内的内容；区域外的内容保持不变
- 如果标记区域不存在，将其追加到文件末尾
- 同步后提示用户运行 `source ~/.zshrc`以生效

---

## 未来规划

- **连通性测试（暂缓）**：通过发送测试请求验证 API Key 和 Base URL。设计待定。
- **更多模块**：侧边栏和路由架构支持添加新的工具模块，无需更改架构。
