# Architecture

## 当前架构

- Rust 平台层：`src-tauri/src/lib.rs`
  - 窗口管理、Tray、全局快捷键、命令注册（10 个命令）。
- 命令层：
  - `src-tauri/src/commands/app.rs`：基础命令（ping, get_app_info）
  - `src-tauri/src/commands/settings.rs`：settings 读写
  - `src-tauri/src/commands/profiles.rs`：Profile CRUD + .zshrc 同步（标记符 `CLAUDE_CODE_ALIAS_START/END`）
- 数据层：`src-tauri/src/db.rs` + `src-tauri/migrations/`
  - SQLite `toolkit.db`，表：`app_settings`、`profiles`
  - `profiles.env_vars` 以 JSON text 存储
- 前端 IPC 层：`src/core/ipc.ts` + `src/core/ipc.generated.ts`
  - `typedInvoke` + 自动生成契约。
- 前端路由层：react-router-dom（BrowserRouter, basename `/main.html`）
  - `src/components/layout/AppLayout.tsx`：Sidebar + Outlet 布局
  - `src/components/layout/Sidebar.tsx`：导航栏
- 前端模块层：
  - `src/modules/app/*`、`src/modules/settings/*`：模板 API
  - `src/modules/profiles/*`：profiles API + TanStack Query hooks
- 页面层：
  - `src/pages/ClaudeConfigPage.tsx`：配置管理页面
  - `src/pages/claude-config/*`：ProfileList、ProfileDialog、DeleteConfirmDialog
- UI 层：`src/windows/main/App.tsx`（路由） + `src/windows/timer/App.tsx`（占位）

## 何时更新本文件

- 模块边界变化（新增/删除核心目录或职责迁移）。
- 命令层、数据层、IPC 层关系发生变化。
- 窗口机制、Tray 或快捷键机制出现结构性变化。
