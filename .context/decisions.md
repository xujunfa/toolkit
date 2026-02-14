# Decision Log

## 已确认决策

1. 保留模板最小命令集：`ping/get_app_info/get_settings/set_settings/update_tray_title`。
2. 数据库采用新文件 `toolkit.db`，不在旧业务库上做 drop 迁移。
3. 前端业务模型与状态不再保留，保持模板中立语义。
4. 全局快捷键使用 `Cmd+Shift+O`（timer）和 `Cmd+Shift+L`（main），避免与旧项目冲突。
5. IPC 契约以 `src/core/ipc.generated.ts` 为唯一事实源。
6. `profiles.env_vars` 以 JSON text 存储（`[{"key":"...","value":"..."}]`），Rust 端序列化/反序列化。
7. 前端路由使用 react-router-dom，BrowserRouter basename `/main.html`。
8. .zshrc 同步采用标记区域方案（`CLAUDE_CODE_ALIAS_START`/`END`），写入前备份 `.zshrc.bak`。
9. `sync_profiles_to_zshrc` 在 Rust 后端直接操作文件系统，无需 Tauri fs plugin 权限。

## 何时更新本文件

- 出现“会影响后续实现方式”的新技术取舍时。
- 需要记录“为什么这么做”且未来可能被质疑时。
- 决策变更时保留“旧决策 -> 新决策”的替换说明。
