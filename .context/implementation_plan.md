# Claude Code Config Manager Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Build a sidebar-navigated toolkit app with a Claude Code profile manager that generates shell aliases into `~/.zshrc`.

**Architecture:** Main window gets react-router with a sidebar layout. Profiles are stored in SQLite via Rust commands, exposed through typed IPC. The `.zshrc` sync is a dedicated Rust command that reads all profiles and writes a marked section.

**Tech Stack:** Tauri v2, React 19, react-router-dom, shadcn/ui, TanStack Query, Jotai, SQLite (sqlx), Tailwind CSS v4

---

## Task 1: Install react-router-dom

**Files:**
- Modify: `package.json`

**Step 1: Install dependency**

Run: `pnpm add react-router-dom`

**Step 2: Verify TypeScript resolves it**

Run: `pnpm -s tsc -b --pretty false`
Expected: PASS (no errors)

**Step 3: Commit**

```bash
git add package.json pnpm-lock.yaml
git commit -m "chore: add react-router-dom"
```

---

## Task 2: Sidebar layout + router shell

**Files:**
- Create: `src/components/layout/Sidebar.tsx`
- Create: `src/components/layout/AppLayout.tsx`
- Create: `src/pages/ClaudeConfigPage.tsx` (placeholder)
- Modify: `src/windows/main/App.tsx`
- Modify: `src/windows/main/main.tsx`

**Step 1: Create Sidebar component**

`src/components/layout/Sidebar.tsx` — a fixed left sidebar using shadcn styling. Uses `NavLink` from react-router-dom. Items array: `[{ to: "/claude-config", icon: Terminal, label: "Claude Config" }]`. Active item gets `bg-accent` class.

**Step 2: Create AppLayout**

`src/components/layout/AppLayout.tsx` — flex row: `<Sidebar />` on left (w-56, fixed), `<Outlet />` on right (flex-1, overflow-auto).

**Step 3: Create placeholder page**

`src/pages/ClaudeConfigPage.tsx` — simple heading "Claude Code Profiles", no logic yet.

**Step 4: Wire router into main window**

Modify `src/windows/main/main.tsx`: wrap with `BrowserRouter` (basename `/main.html`).
Modify `src/windows/main/App.tsx`: replace current content with `<Routes>` using `<AppLayout>` as layout route, `<ClaudeConfigPage>` at `/claude-config`, redirect `/` to `/claude-config`.

**Step 5: Verify**

Run: `pnpm -s tsc -b --pretty false`
Expected: PASS

Run: `pnpm test`
Expected: PASS

**Step 6: Commit**

```bash
git add src/components/layout/ src/pages/ src/windows/main/
git commit -m "feat: add sidebar layout with react-router"
```

---

## Task 3: Profiles migration + Rust struct

**Files:**
- Create: `src-tauri/migrations/002_profiles.sql`
- Modify: `src-tauri/src/db.rs`
- Create: `src-tauri/src/commands/profiles.rs`
- Modify: `src-tauri/src/commands/mod.rs`

**Step 1: Write migration SQL**

`src-tauri/migrations/002_profiles.sql`:
```sql
CREATE TABLE IF NOT EXISTS profiles (
  id TEXT PRIMARY KEY,
  name TEXT NOT NULL,
  alias TEXT NOT NULL UNIQUE,
  env_vars TEXT NOT NULL DEFAULT '[]',
  created_at TEXT NOT NULL DEFAULT (datetime('now')),
  updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);
```

`env_vars` is JSON text: `[{"key":"ANTHROPIC_BASE_URL","value":"..."},...]`

**Step 2: Register migration**

In `src-tauri/src/db.rs`, add second `Migration` entry (version 2, sql = `include_str!("../migrations/002_profiles.sql")`).

**Step 3: Create profiles command file with struct definitions only**

`src-tauri/src/commands/profiles.rs`:
- `Profile` struct (id, name, alias, env_vars as `String`, created_at, updated_at) deriving `Serialize, Deserialize, sqlx::FromRow`
- `EnvVar` struct (key, value) deriving `Serialize, Deserialize`
- `CreateProfileInput` (name, alias, env_vars: `Vec<EnvVar>`)
- `UpdateProfileInput` (name: Option, alias: Option, env_vars: Option)

Register module in `src-tauri/src/commands/mod.rs`: add `pub mod profiles;`.

**Step 4: Verify Rust compiles**

Run: `cargo build --manifest-path src-tauri/Cargo.toml`
Expected: PASS

**Step 5: Commit**

```bash
git add src-tauri/migrations/002_profiles.sql src-tauri/src/db.rs src-tauri/src/commands/profiles.rs src-tauri/src/commands/mod.rs
git commit -m "feat: add profiles table migration and Rust structs"
```

---

## Task 4: Rust CRUD commands — list + create

**Files:**
- Modify: `src-tauri/src/commands/profiles.rs`
- Modify: `src-tauri/src/lib.rs`

**Step 1: Write failing test for list_profiles**

In `profiles.rs` add `#[cfg(test)] mod tests` with `setup_db()` (memory SQLite, create profiles table), then test `list_profiles_by_pool` returns empty vec.

**Step 2: Run test to verify it fails**

Run: `cargo test --manifest-path src-tauri/Cargo.toml profiles`
Expected: FAIL (function not defined)

**Step 3: Implement `list_profiles_by_pool` and `list_profiles`**

`list_profiles_by_pool(db)` — `SELECT * FROM profiles ORDER BY created_at DESC`, parse `env_vars` JSON text into `Vec<EnvVar>` after fetch, return `Vec<ProfileResponse>` (with env_vars as `Vec<EnvVar>` instead of raw string).

Define `ProfileResponse` struct (id, name, alias, env_vars: Vec<EnvVar>, created_at, updated_at).

`#[tauri::command] list_profiles(db: State<SqlitePool>)` — calls pool function.

**Step 4: Run test to verify it passes**

Run: `cargo test --manifest-path src-tauri/Cargo.toml profiles`
Expected: PASS

**Step 5: Write failing test for create_profile**

Test: create a profile with name="Leo", alias="ccleo", env_vars=[{key: "ANTHROPIC_BASE_URL", value: "https://example.com"}]. Then list_profiles should return 1 item with matching fields.

**Step 6: Run test, verify fail**

Run: `cargo test --manifest-path src-tauri/Cargo.toml profiles`
Expected: FAIL

**Step 7: Implement `create_profile_by_pool` and `create_profile`**

Generate UUID via `format!("{}", uuid)` — use `sqlx::types::Uuid` or simply generate with timestamp+random. Simpler: add `uuid` crate with `v4` feature to `Cargo.toml`. Insert row with `serde_json::to_string(&input.env_vars)` for env_vars column.

**Step 8: Run test, verify pass**

Run: `cargo test --manifest-path src-tauri/Cargo.toml profiles`
Expected: PASS

**Step 9: Register commands in `generate_handler![]`**

In `src-tauri/src/lib.rs`, add `commands::profiles::list_profiles` and `commands::profiles::create_profile` to `generate_handler![]`.

**Step 10: Regenerate IPC types**

Run: `pnpm gen:ipc`
Run: `pnpm gen:ipc:check`
Expected: PASS

**Step 11: Commit**

```bash
git add src-tauri/ src/core/ipc.generated.ts
git commit -m "feat: add list_profiles and create_profile commands"
```

---

## Task 5: Rust CRUD commands — update + delete

**Files:**
- Modify: `src-tauri/src/commands/profiles.rs`
- Modify: `src-tauri/src/lib.rs`

**Step 1: Write failing test for update_profile**

Test: create profile, then update name and alias, verify list returns updated values.

**Step 2: Run test, verify fail**

**Step 3: Implement `update_profile_by_pool` + `update_profile`**

Partial update: merge only provided fields (same pattern as `set_settings`). Update `updated_at` to `datetime('now')`.

**Step 4: Run test, verify pass**

**Step 5: Write failing test for delete_profile**

Test: create profile, delete by id, verify list returns empty.

**Step 6: Run test, verify fail**

**Step 7: Implement `delete_profile_by_pool` + `delete_profile`**

`DELETE FROM profiles WHERE id = ?`, return `()`.

**Step 8: Run test, verify pass**

**Step 9: Register in `generate_handler![]`, regenerate IPC**

Run: `pnpm gen:ipc && pnpm gen:ipc:check`

**Step 10: Full verification**

Run: `cargo test --manifest-path src-tauri/Cargo.toml`
Run: `pnpm -s tsc -b --pretty false`
Run: `pnpm test`
Expected: all PASS

**Step 11: Commit**

```bash
git add src-tauri/ src/core/ipc.generated.ts
git commit -m "feat: add update_profile and delete_profile commands"
```

---

## Task 6: Frontend profiles API module

**Files:**
- Create: `src/modules/profiles/api.ts`
- Create: `src/modules/profiles/index.ts`
- Create: `src/modules/profiles/queries.ts`

**Step 1: Create API wrapper**

`src/modules/profiles/api.ts`:
- `listProfiles()` — `typedInvoke('list_profiles', {})`
- `createProfile(input)` — `typedInvoke('create_profile', { input })`
- `updateProfile(id, input)` — `typedInvoke('update_profile', { id, input })`
- `deleteProfile(id)` — `typedInvoke('delete_profile', { id })`

**Step 2: Create TanStack Query hooks**

`src/modules/profiles/queries.ts`:
- `useProfiles()` — `useQuery({ queryKey: ['profiles'], queryFn: listProfiles })`
- `useCreateProfile()` — `useMutation` + invalidate `['profiles']`
- `useUpdateProfile()` — `useMutation` + invalidate `['profiles']`
- `useDeleteProfile()` — `useMutation` + invalidate `['profiles']`

**Step 3: Create barrel export**

`src/modules/profiles/index.ts` — re-export api + queries.

**Step 4: Verify**

Run: `pnpm -s tsc -b --pretty false`
Expected: PASS

**Step 5: Commit**

```bash
git add src/modules/profiles/
git commit -m "feat: add frontend profiles API and query hooks"
```

---

## Task 7: Profile list UI

**Files:**
- Modify: `src/pages/ClaudeConfigPage.tsx`
- Create: `src/pages/claude-config/ProfileList.tsx`

**Step 1: Build ProfileList component**

Table layout using shadcn `<Table>` (add via `npx shadcn@latest add table` if needed). Columns: Name, Alias, Env Vars count, Actions (edit/delete buttons). API keys masked: if value starts with `sk-` or contains "key"/"token" (case-insensitive), display as `sk-****`.

Uses `useProfiles()` hook. Loading and empty states.

**Step 2: Wire into page**

`ClaudeConfigPage.tsx`: render header + "Add Profile" button + `<ProfileList />`.

**Step 3: Verify**

Run: `pnpm -s tsc -b --pretty false`
Expected: PASS

**Step 4: Commit**

```bash
git add src/pages/
git commit -m "feat: add profile list UI with masking"
```

---

## Task 8: Profile add/edit dialog

**Files:**
- Create: `src/pages/claude-config/ProfileDialog.tsx`
- Modify: `src/pages/ClaudeConfigPage.tsx`

**Step 1: Build ProfileDialog component**

Uses existing shadcn `<Dialog>`, `<Input>`, `<Button>`. Props: `open`, `onOpenChange`, `profile?` (for edit mode).

Form fields:
- Name (text input)
- Alias (text input, auto-prefix hint "cc")
- Env vars: dynamic rows with key + value inputs, add/remove row buttons

On submit: call `useCreateProfile` or `useUpdateProfile` based on mode. Close dialog on success.

Use Jotai atom for dialog state: `profileDialogAtom` = `{ open: boolean, profile: Profile | null }`.

**Step 2: Wire into page**

`ClaudeConfigPage.tsx`: "Add Profile" button sets dialog open. Edit button in list sets dialog with profile data.

**Step 3: Add delete confirmation**

Use shadcn `<Dialog>` for confirm. On confirm call `useDeleteProfile`.

**Step 4: Verify**

Run: `pnpm -s tsc -b --pretty false`
Run: `pnpm test`
Expected: PASS

**Step 5: Commit**

```bash
git add src/pages/claude-config/ src/pages/ClaudeConfigPage.tsx
git commit -m "feat: add profile add/edit/delete dialogs"
```

---

## Task 9: Rust sync_profiles_to_zshrc command

**Files:**
- Modify: `src-tauri/src/commands/profiles.rs`
- Modify: `src-tauri/src/lib.rs`
- Modify: `src-tauri/Cargo.toml` (if `dirs` crate needed)

**Step 1: Write failing test for alias generation**

Test a pure function `generate_alias_block(profiles) -> String` that returns:
```
# === CLAUDE_CODE_ALIAS_START (DO NOT EDIT MANUALLY) ===
alias ccleo="ANTHROPIC_BASE_URL=https://example.com ANTHROPIC_AUTH_TOKEN=sk-xxx claude"
# === CLAUDE_CODE_ALIAS_END ===
```

**Step 2: Run test, verify fail**

**Step 3: Implement `generate_alias_block`**

Pure function: iterates profiles, for each builds `alias {alias}="{KEY=VAL KEY=VAL ...} claude"`, wraps in start/end markers.

**Step 4: Run test, verify pass**

**Step 5: Write failing test for zshrc content replacement**

Test a pure function `replace_marker_section(existing_content, new_block) -> String`:
- Case 1: markers exist — replace content between them
- Case 2: no markers — append to end

**Step 6: Run test, verify fail**

**Step 7: Implement `replace_marker_section`**

String manipulation: find start marker line, find end marker line, replace range. If not found, append `\n{block}\n` to end.

**Step 8: Run test, verify pass**

**Step 9: Implement `sync_profiles_to_zshrc` command**

- Read all profiles from DB
- Generate alias block
- Read `~/.zshrc` (use `dirs::home_dir()` — add `dirs` crate)
- Backup to `~/.zshrc.bak` (fs::copy)
- Replace marker section
- Write back
- Return success message string

**Step 10: Register in `generate_handler![]`, regenerate IPC**

Run: `pnpm gen:ipc && pnpm gen:ipc:check`

**Step 11: Full verification**

Run: `cargo test --manifest-path src-tauri/Cargo.toml`
Run: `pnpm -s tsc -b --pretty false`
Expected: PASS

**Step 12: Commit**

```bash
git add src-tauri/ src/core/ipc.generated.ts
git commit -m "feat: add sync_profiles_to_zshrc command"
```

---

## Task 10: Frontend sync button + notification

**Files:**
- Modify: `src/pages/ClaudeConfigPage.tsx`
- Modify: `src/modules/profiles/api.ts`
- Modify: `src/modules/profiles/queries.ts`

**Step 1: Add API wrapper**

In `api.ts`: `syncProfilesToZshrc()` — `typedInvoke('sync_profiles_to_zshrc', {})`.

**Step 2: Add mutation hook**

In `queries.ts`: `useSyncToZshrc()` — `useMutation`, on success show notification/toast: "Aliases synced! Run `source ~/.zshrc` to apply."

**Step 3: Add sync button to page**

In `ClaudeConfigPage.tsx`: "Sync to .zshrc" button in header area, calls `useSyncToZshrc`. Show loading state during sync. Display success message inline or via notification.

**Step 4: Verify**

Run: `pnpm -s tsc -b --pretty false`
Run: `pnpm test`
Expected: PASS

**Step 5: Commit**

```bash
git add src/modules/profiles/ src/pages/
git commit -m "feat: add sync to zshrc button with notification"
```

---

## Task 11: Tauri permissions for fs access

**Files:**
- Modify: `src-tauri/capabilities/default.json`

**Step 1: Add fs permissions**

The `sync_profiles_to_zshrc` command runs in Rust backend (not frontend fs plugin), so it has native fs access. No additional Tauri capability needed — verify this by testing.

If `tauri-plugin-fs` scoping is needed for any reason, add `fs:default` and scope `$HOME/.zshrc` to capabilities.

**Step 2: Verify end-to-end**

Run: `pnpm gen:ipc:check && pnpm -s tsc -b --pretty false && pnpm test && cargo test --manifest-path src-tauri/Cargo.toml`
Expected: all PASS

**Step 3: Commit (if changes needed)**

```bash
git add src-tauri/capabilities/
git commit -m "chore: add fs permissions for zshrc access"
```

---

## Task 12: Update project docs

**Files:**
- Modify: `.claude/CLAUDE.md` — add new commands, update module list
- Modify: `.context/architecture.md` — add profiles module, sidebar layout, router
- Modify: `.context/decisions.md` — record new decisions (env_vars as JSON, marker-based zshrc sync)
- Modify: `.context/active_context.md` — update current status

**Step 1: Update docs to reflect new architecture**

Key updates:
- CLAUDE.md: add profile commands to command list, add `src/pages/` and `src/modules/profiles/` to key directories
- architecture.md: add router layer, sidebar layout, profiles module
- decisions.md: env_vars stored as JSON text, react-router for navigation, marker-based zshrc strategy

**Step 2: Commit**

```bash
git add .claude/ .context/
git commit -m "docs: update project docs for claude config module"
```

## 验收命令（全 Task 通用）

```bash
pnpm gen:ipc:check
pnpm -s tsc -b --pretty false
pnpm test
cargo test --manifest-path src-tauri/Cargo.toml
```

## 何时更新本文件

- 开始新 Task 实施前。
- Task 内步骤拆分发生明显变化时。
- 验收标准或执行顺序调整时。
