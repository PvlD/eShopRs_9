# webapprsa3 — Leptos + Axum (Workspace)

## Toolchain
- Rust **nightly** (set in `rust-toolchain.toml`)
- WASM target: `rustup target add wasm32-unknown-unknown`
- Install `cargo-leptos` before building: `cargo install cargo-leptos --locked`

## Cargo Workspace — 3 packages

| Package | Type | Role | Cargo features |
|---------|------|------|----------------|
| `app/`  | lib  | Shared UI components, routing, pages | `ssr` (server), `hydrate` (client) |
| `server/` | bin | Axum HTTP server, entrypoint | depends on `app` with `ssr` |
| `frontend/` | lib (cdylib+rlib) | WASM hydrate entrypoint | depends on `app` with `hydrate` |

`app/src/lib.rs` uses `cfg_if!` or `#[cfg(feature = "ssr")]` to gate server-only code (Axum extractors, DB access, etc.).

## Build & dev commands

| Task | Command |
|------|---------|
| Build Tailwind CSS | `npm run css` |
| Watch Tailwind CSS | `npm run css:watch` |
| Dev server (auto-reload) | `cargo leptos watch` (requires `npm run css` first) |
| Release build | `cargo leptos build --release` (requires `npm run css` first) |
| Build single package | `cargo build --package <server\|frontend>` |
| E2E tests (dev) | `cargo leptos end-to-end` |
| E2E tests (release) | `cargo leptos end-to-end --release` |
| Show E2E report | `cd end2end && npx playwright show-report` |

**Dev workflow:** Terminal 1 → `npm run css:watch`, Terminal 2 → `cargo leptos watch`.

## Flicker prevention (client navigation)

- `transition=true` on `<Routes>` — uses View Transition API (`document.startViewTransition`) to keep current page visible while new route data loads, then crossfades on swap. Covers all client-side `<a>` navigation.
- No `SsrMode::InOrder` or `OutOfOrder` on individual routes — initial full page load may show a brief empty-then-populated transition, which is acceptable.
- Checkout and cart pages use `Resource` + `Effect::new` pattern (no `<Transition>`/`<Suspense>` boundary) to avoid blank fallback during hydration.

## Important conventions

- The `app` crate **must** be imported in `frontend/src/lib.rs` (`use app;`) for islands/wasm-bindgen to work correctly. Suppress clippy with `#[allow(clippy::single_component_path_imports)]`.
- CSS source: `style/app.css` (Tailwind v4 entry via `@import "tailwindcss"`). Output: `style/main.css` (compiled by Tailwind CLI → Lightning CSS).
- Assets: `public/` → copied verbatim to site root.
- E2E tests live in `end2end/tests/`, written in Playwright (TypeScript), expect server at `http://localhost:3030/`.
- No CI, no formatter/lint config — uses standard `cargo fmt` / `cargo clippy`.
