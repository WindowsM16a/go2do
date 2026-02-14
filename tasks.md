# Go2Do V2 Implementation Tasks

## Phase 1: Authentication & Server Foundation

_Setup robust auth and database schema to support multi-user sync._

- [x] **Database Schema Update** <!-- id: 0 -->
  - Update `schema.sql` to include `users` table (`id`, `email`, `password_hash`, `created_at`).
  - Add indexes: `idx_tasks_user_updated` (`user_id`, `updated_at`), `idx_tasks_user_deleted` (`user_id`, `deleted_at`).
  - Run migrations on D1.
- [ ] **Auth Endpoints (Worker)** <!-- id: 1 -->
  - _Constraint_: Must not require an account for local use (Retention Rule).
  - Install `bcryptjs` and `jose`.
  - Implement `POST /auth/register`: Simple Email/Password. No email verification strictly required for V1 (reduces friction), just format check.
  - Implement `POST /auth/login`: Return long-lived `HttpOnly` cookie + Refresh Token.
  - _Security_: Rate limit login attempts to prevent brute force.
- [ ] **KV Setup** <!-- id: 2 -->
  - Provision KV namespace `go2do-kv`.
  - Update `wrangler.toml` to bind KV.

## Phase 2: Optimized Sync Protocol (Server)

_Implement the low-cost sync logic to stay within free tier limits._

- [ ] **Refactor `POST /sync`** <!-- id: 3 -->
  - Update to handle atomic push-then-pull.
  - Ensure `db.batch()` is used for all writes (1 write op).
  - Implement conflict resolution: `WHERE excluded.version > tasks.version`.
- [ ] **Implement `GET /sync/check`** <!-- id: 4 -->
  - Logic: Single query `SELECT COUNT(*), MAX(updated_at), SUM(version) FROM tasks WHERE user_id = ?`.
  - Return `ETag` header.
  - If `If-None-Match` matches, return `304 Not Modified`.
- [ ] **Implement KV Caching** <!-- id: 5 -->
  - Wrap `/sync/check`: Check KV `hash:{userId}` first.
  - Cache Hit: Return 304 immediately (0 D1 reads).
  - Cache Miss: Query D1, update KV (TTL 5 mins), return ETag.
- [ ] **Rate Limiting** <!-- id: 6 -->
  - Implement Token Bucket in KV (`ratelimit:{userId}`).
  - Limit: 20 requests/minute (burst), 100 requests/day (soft cap).
  - Return `429` if exceeded.

## Phase 3: Desktop Client Core (Rust)

_Upgrade client logic to handle smart syncing and authentication._

- [ ] **Auth Logic** <!-- id: 7 -->
  - Create `AuthManager` struct.
  - **Retention Strategy**: App starts _Offline_ by default. No login screen on launch.
  - Store JWT in OS Keyring.
  - UI: "Connect Sync" button in Settings/Tray. Only then show Login/Register dialog.
  - _Seamless_: Generate a random "Device Name" automatically to reduce input fields.
- [ ] **Smart Sync Implementation** <!-- id: 8 -->
  - Refactor `sync.rs` into `SyncManager`.
  - Implement `SyncState`: `Online`, `Offline`, `Degraded` (429).
  - Logic: `check_server_hash()` before full sync.
  - Logic: `ExponentialBackoff` for polling (1m -> 5m -> 15m).
- [ ] **Sync Triggers** <!-- id: 9 -->
  - **Debounce**: Wait 5s after last local edit before pushing.
  - **Activity**: Pause background sync if inactive > 1 hour (unless "Offline Hours" logic used).

## Phase 4: Desktop UI Polish (GTK4)

_Implement the "Minimalist Black" design system._

- [ ] **Global CSS Provider** <!-- id: 10 -->
  - Port CSS from `go2do_ui1.md` to GTK CSS.
  - Define colors: `#0a0a0a` (bg), `#ffffff` (text), `#22c55e` (green), etc.
  - Apply `Geist` font (bundled or system fallback).
- [ ] **Quick Input Window** <!-- id: 11 -->
  - Create `QuickInputWindow` struct.
  - Logic: Global hotkey (e.g., `Ctrl+Space` using `global-hotkey` crate) to toggle visibility.
  - Animation: Opacity/Scale fade-in.
- [ ] **Task List UI** <!-- id: 12 -->
  - Update `MainWindow` to remove native titlebar (use custom header).
  - Implement `TaskRow` widget:
    - Checkbox with custom styling.
    - Hover actions (Pin, Edit, Delete).
    - Strikethrough animation.
  - **Completed Tasks View**:
    - Implement "Show/Hide Completed" toggle at bottom of list.
    - Section separator: "Completed Today".
  - Status Bar: Show "Synced X min ago" with colored dots.
- [ ] **System Tray** <!-- id: 13 -->
  - Implement `ksni` or `tray-icon` crate.
  - Menu: "Add Task", "Today", "Sync Now", "Quit".
  - Icon: Dynamic color based on sync status (Green=Synced, Red=Offline).

## Phase 5: Web Application (Next.js)

_Create the V2 web interface._

- [ ] **Project Setup** <!-- id: 14 -->
  - `npx create-next-app` with Tailwind.
  - config: `tailwind.config.js` to match `go2do_ui1.md` colors.
- [ ] **Auth Pages** <!-- id: 15 -->
  - `/login` and `/register`.
  - Match design: Centered card, minimal inputs.
- [ ] **Main App Interface** <!-- id: 16 -->
  - Replicate Desktop Layout: Sidebar (optional/mobile), Main List, Quick Input.
  - **State Management**: Use `React Query` or `SWR` for sync state.
  - **Offline Support**: `next-pwa` for Service Worker caching.
- [ ] **Web Sync Logic** <!-- id: 17 -->
  - Port `SyncManager` logic to React hooks.
  - Use `localStorage` or `IndexedDB` for offline capability.

## Phase 6: Distribution & Launch

_Prepare for release._

- [ ] **Linux Packaging** <!-- id: 18 -->
  - Update `install.sh`.
  - Create `deb` or `AppImage` (optional, current script builds source).
- [ ] **Windows Support** <!-- id: 19 -->
  - Setup `cross` for cross-compilation: `cargo install cross`.
  - Target: `x86_64-pc-windows-gnu` (easier from Linux than msvc).
  - Create `wix` installer config (`wix` toolset required) or simple `zip` release.
- [ ] **macOS Support** <!-- id: 21 -->
  - _Note_: Hard to build from Linux.
  - Setup GitHub Action matrix to build on `macos-latest` runner.
  - Bundle `.app` structure.
- [ ] **Documentation** <!-- id: 20 -->
  - Update `README.md` with V2 features.
  - Add "Hotkeys" help section in app.
