# Active Context

## 当前状态（更新于 2026-02-14）

- Claude Config Manager：全部完成，待手动端到端验证。
- **下一步：ZenMux Quota Monitor**（第二个工具模块）。
- 设计文档：`.context/zenmux-quota-design.md`。

## ZenMux Quota Monitor（待执行）

- [ ] Task 1: Rust ZenMux HTTP 客户端（reqwest + Cookie 认证）
- [ ] Task 2: Rust settings 读写（get/set_zenmux_config）
- [ ] Task 3: Rust 轮询服务（tokio interval + AppState）
- [ ] Task 4: Rust macOS Tray FFI（objc2 两行 attributedTitle）
- [ ] Task 5: Rust 命令注册（5 个新命令 → generate_handler）
- [ ] Task 6: 前端 IPC 生成（pnpm gen:ipc）
- [ ] Task 7: 前端 ZenMux API 层 + TanStack Query hooks
- [ ] Task 8: 前端配置页面 UI（Cookie 输入 + 状态展示）
- [ ] Task 9: 前端侧边栏路由（新增导航项）
- [ ] Task 10: 验收（tsc + test + cargo test + 手动验证 Tray）

## Claude Config Manager（已完成）

- [x] Task 1~12: 全部完成

## 何时更新本文件

- 每完成一个阶段（Phase）后更新一次。
- 发生"可继续工作的上下文变化"时更新。
- 每次更新保持 5-15 行，避免写流水账。
