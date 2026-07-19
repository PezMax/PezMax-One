# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build & Run

```bash
cargo build              # debug build
cargo build --release    # release build
cargo check              # fast type-check only
cargo run                # run the desktop app
cargo fix                # auto-fix warnings
```

The app is a native Windows desktop binary (egui/eframe). No external build tooling needed.

## Submodule

```bash
git submodule update --init --recursive   # clone PezMax-Java after first checkout
cd PezMax-Java && git checkout main       # ensure submodule is on the right branch
```

The Java backend is a separate project (`PezMax-Java/`) — this repo only contains the Rust frontend. The API contract is documented in `后端接口列表.md`.

## Architecture

### Monorepo Layout

```
PezMax-One/                  ← product root, Rust crate root
├── src/
│   ├── main.rs              ← eframe entry, window config
│   ├── app.rs               ← PezMaxApp state, routing, eframe::App impl
│   ├── api/                 ← typed HTTP client (reqwest)
│   │   ├── client.rs        ← ApiClient: GET/POST/PUT/DELETE/upload/download
│   │   ├── models.rs        ← 28 serde models matching backend JSON
│   │   ├── auth.rs          ← login, register, captcha, password reset
│   │   ├── file.rs          ← paper file CRUD, tree, search
│   │   ├── bookmark.rs      ← bookmark CRUD
│   │   ├── user.rs          ← profile, avatar, password, security
│   │   ├── download.rs      ← download records, favorites
│   │   ├── notification.rs  ← popup/scroll notifications
│   │   └── report.rs        ← report creation & timeline
│   ├── theme/
│   │   └── mod.rs           ← Metro Design colors, fonts, spacing
│   ├── components/
│   │   ├── sidebar.rs       ← dark sidebar with nav + badges + logout
│   │   ├── topbar.rs        ← title, search, avatar, back button
│   │   └── toast.rs         ← non-intrusive corner notifications
│   └── pages/
│       ├── mod.rs           ← 12 page modules
│       ├── login.rs         ← Metro login card
│       ├── register.rs      ← 3-step registration flow
│       ├── forget_password.rs
│       ├── home.rs          ← Metro tile dashboard
│       ├── file_explorer.rs ← file tree + card grid
│       ├── file_detail.rs
│       ├── bookmarks.rs
│       ├── downloads.rs
│       ├── favorites.rs
│       ├── notifications.rs
│       ├── profile.rs
│       ├── security.rs      ← password + security questions
│       ├── report.rs
│       └── settings.rs
├── PezMax-Java/             ← git submodule (Java Spring Boot backend)
├── PezMax-Desktop/          ← reference: old Electron+Vue3 frontend (gitignored)
├── repowiki/                ← knowledge base (tracked for reference)
├── resources/icon.png       ← app icon
└── 后端接口列表.md           ← full API contract for all 34 backend controllers
```

### Key Design Decisions

- **egui immediate mode**: UI is rebuilt every frame. No hidden/shown state — conditional rendering via `match` on `current_page`.
- **Single state struct**: `PezMaxApp` holds all app state. Pages are pure functions `fn render(&mut PezMaxApp, &mut Ui)`. No per-page state.
- **API via trait extension**: `ApiClient` methods are defined across multiple files via `impl ApiClient { ... }` blocks, one per domain.
- **Metro Design**: Flat colors, large typography, generous whitespace, content-first cards. Theme defined in `theme/mod.rs` as constants.
- **Async HTTP**: `reqwest` is used. The eframe frame loop is synchronous — async calls use `tokio::spawn` + channels to feed results back into the frame.

### API Layer (mapping to 后端接口列表.md)

| Module | Backend base path | Key endpoints |
|--------|------------------|---------------|
| `auth` | `/datum/user` | login, register, captcha, securityQuestions, resetPasswordBySecurity |
| `file` | `/datum/file` | list, tree, subjects, schools, search, CRUD, /datum/download/file |
| `bookmark` | `/datum/bookmark` | CRUD, uploadCover |
| `user` | `/datum/desktop/user/profile` | stats, username, avatar, password, security |
| `download` | `/datum/desktop/download` | list, hide; favorites via /datum/desktop/favorite |
| `notification` | `/system/notification/user` | popup, scroll |
| `report` | `/datum/report` | create, timeline |

### Page Routing

Pages are split into two groups in `app.rs`:
- **Auth pages** (Login, Register, ForgetPassword) — rendered full-screen when `is_logged_in == false`
- **App pages** (Home, FileExplorer, etc.) — rendered inside sidebar + topbar + central panel when logged in

Navigation is push-history: `navigate(page)` pushes current page, `go_back()` pops.

### Theme System

`theme/colors` module provides ~20 constants. `apply_metro_theme()` sets egui's `Style`: text sizes, spacing, corner radius, colors. All pages import `colors::*` for consistency.

### Current State

The project is a skeleton with all page/routing/API scaffolding in place, but most pages use mock data. High-priority next steps:
1. Replace mock login with real API calls in `login.rs`
2. Wire up `file_explorer.rs` to `get_file_list()` / `get_file_tree()`
3. Implement actual API calls in download, favorite, bookmark, notification pages
4. Add file download via `rfd` (save dialog) + `reqwest` streaming