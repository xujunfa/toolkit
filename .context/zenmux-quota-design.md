# ZenMux Quota Monitor 设计文档

## 概述

第二个工具模块：轮询抓取 ZenMux 订阅账号的额度信息，在 macOS 菜单栏 Tray 上以两行紧凑文本实时显示。

## 数据源

- **API**：`GET https://zenmux.ai/api/subscription/get_current_usage?ctoken={ctoken}`
- **认证**：Cookie 中需要 `ctoken`、`sessionId`、`sessionId.sig` 三个字段，`ctoken` 同时作为 query param

### curl 拨测

```bash
curl -s "https://zenmux.ai/api/subscription/get_current_usage?ctoken={ctoken}" \
  -H "accept: application/json" \
  -H "cookie: ctoken={ctoken}; sessionId={sessionId}; sessionId.sig={sessionId.sig}"
```

实测示例（2026-02-14 已验证可通，HTTP 200）：

```bash
curl -s "https://zenmux.ai/api/subscription/get_current_usage?ctoken=wkukHWvxr9IwQTkaX5nWJLFg" \
  -H "accept: application/json" \
  -H "cookie: ctoken=wkukHWvxr9IwQTkaX5nWJLFg; sessionId=ae044998-7f12-4364-9455-51e6f6d2026e; sessionId.sig=t5hjYqfWekfHWKmD3lly-zQiIGcOEe_neBhadNuzLEk"
```

> 注意：`sessionId` / `sessionId.sig` 是有时效的 session cookie，过期后需要重新从浏览器 DevTools 获取。

- **响应结构**：

```json
{
  "success": true,
  "data": [
    {
      "tierCode": "max",
      "periodType": "week",
      "periodDuration": "168",
      "cycleStartTime": "2026-02-07T00:40:05.000Z",
      "cycleEndTime": "2026-02-14T00:40:05.000Z",
      "usedRate": 0.6198,
      "quotaStatus": 0,
      "status": 0
    },
    {
      "tierCode": "max",
      "periodType": "hour_5",
      "periodDuration": "5",
      "cycleStartTime": "2026-02-13T16:13:06.000Z",
      "cycleEndTime": "2026-02-13T21:13:06.000Z",
      "usedRate": 0.2354,
      "quotaStatus": 0,
      "status": 0
    }
  ]
}
```

- `usedRate`：已用比例（0~1），剩余 = 1 - usedRate
- `cycleEndTime`：窗口刷新时间，与当前时间差即为倒计时

## Tray 显示

### 格式（两行紧凑文本）

```
5h:76% 3h12m
 W:38%  5d8h
```

- 第一行：5h 窗口剩余百分比 + 距刷新倒计时
- 第二行：周窗口剩余百分比 + 距重置倒计时
- 未配置或请求失败时显示 `ZM: --`

### 实现方式

Tauri `set_title()` 只支持纯文本单行。需通过 Rust FFI（`objc2` crate）直接操作 macOS 原生 API：

1. 获取 `NSStatusItem` 的 `button`（`NSStatusBarButton`）
2. 构造 `NSAttributedString`，包含 `\n` 换行
3. 设置 `NSMutableParagraphStyle`：字号 ~9pt，`lineSpacing=0`，紧凑 `maximumLineHeight`
4. 调用 `button.setAttributedTitle()`

封装为 Rust 函数 `update_tray_quota(app, hour5_text, week_text)`。

## Rust 后端

### 新增命令

| 命令 | 说明 |
|------|------|
| `get_zenmux_config` | 读取 ZenMux Cookie 配置 |
| `set_zenmux_config` | 保存 ZenMux Cookie 配置到 SQLite |
| `get_zenmux_usage` | 手动触发一次 API 抓取，返回额度数据 |
| `start_zenmux_polling` | 启动后台轮询（默认 60s 间隔） |
| `stop_zenmux_polling` | 停止后台轮询 |

### 轮询机制

- App 启动 → 读取 settings 中的 Cookie → 若有效则自动启动轮询
- `tokio::time::interval(Duration::from_secs(60))` 定时请求
- 每次请求成功 → 计算剩余% + 倒计时 → FFI 更新 Tray → `emit("zenmux-usage-updated", data)` 推送前端
- 请求失败 → Tray 显示 `ZM: --`，不中断轮询

### 依赖

- `reqwest`：HTTP 请求（已在 Cargo.toml 或新增）
- `objc2` / `objc2-app-kit`：macOS FFI 操作 NSStatusBarButton
- `chrono`：时间计算

## 前端页面

### 侧边栏

新增导航项 **"ZenMux Quota"**，路由 `/zenmux-quota`。

### 页面内容

**配置区**：
- 文本框粘贴完整 Cookie 字符串（包含 ctoken、sessionId、sessionId.sig）
- 保存按钮 → `set_zenmux_config` 存入 SQLite

**状态区**（调试用）：
- 当前 5h / 周额度数值 + 刷新时间
- 轮询状态（运行中 / 已停止）
- 最后更新时间

## 数据流

```
App 启动
  → 读取 settings 中的 Cookie
  → 启动 tokio interval（60s）
  → 每次请求 ZenMux API
  → 解析 JSON → 计算剩余% + 倒计时
  → FFI 更新 Tray attributedTitle（两行）
  → emit("zenmux-usage-updated", data) 推送前端
```

## 实现任务拆解

1. **Rust: ZenMux HTTP 客户端** — reqwest 封装，Cookie 认证，解析响应
2. **Rust: settings 读写** — get/set_zenmux_config 命令，复用 SQLite settings 表
3. **Rust: 轮询服务** — tokio interval + AppState 管理轮询生命周期
4. **Rust: macOS Tray FFI** — objc2 操作 NSStatusBarButton.attributedTitle 两行显示
5. **Rust: 命令注册** — 5 个新命令注册到 generate_handler
6. **前端: IPC 生成** — `pnpm gen:ipc` 生成类型
7. **前端: ZenMux API 层** — typedInvoke 封装 + TanStack Query hooks
8. **前端: 配置页面 UI** — Cookie 输入 + 状态展示
9. **前端: 侧边栏路由** — 新增导航项 + 路由
10. **验收** — tsc + test + cargo test + 手动验证 Tray 效果
