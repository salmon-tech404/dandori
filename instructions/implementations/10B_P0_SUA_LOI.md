# 10B. P0 fixes (T14B) — written in English to save tokens

Source: `reports/REVIEW_P0.md` (independent review by Claude, 2026-10-31). P0 gate is **NOT passed** until this task is DONE and Claude re-reviews.

## T14B. Close the gaps found in the P0 review

- **Depends on:** T14 (commit all P0 work first; the owner does this).
- **Touch:** `crates/dandori-server/**`, `crates/mod-auth-login/**`, `crates/dandori-registry/**` (route gate only), `crates/dandori-process/**`, `crates/dandori-vault/**`, `crates/dandori-e2e/**`, `crates/mod-sample-hello/**`, `docs/manual-test-checklist.md`, report `reports/T14B.md`.
- **Do not:** loosen any quality config (Cargo lints, clippy, deny, eslint, verify.mjs, scripts). Only ADD. No new libraries (`sha2` was already approved by Claude in the review; nothing else). No fake "verified on Linux" claims.

### Work items (in priority order)

1. **Make `dandori` actually serve (BLOCKER).** Today `dandori_server::run()` only logs one line and returns; nothing listens. Implement: load `ServerConfig`; open the SQLite DB in the data dir; run core + module migrations; build vault, registry, lifecycle manager, event bus; register modules from `dandori_modules::registered_modules()`; `restore_active_modules()`; `create_app`; bind `127.0.0.1:<port>`; `axum::serve(...).with_graceful_shutdown(...)`. Shutdown triggers: Ctrl+C everywhere and SIGTERM on unix (`tokio::signal`, behind `#[cfg(unix)]`). On shutdown: stop accepting, wait for in-flight requests (with a deadline), cancel module workers, close DB, return `Ok`. Keep `main.rs` thin. (T10 acceptance criteria 1 and 8b were not met.)
   - Test A: start the real server on port 0, send a real HTTP request over TCP to `/api/v1/health` (use `reqwest` or a raw TCP client from the approved list), expect 200.
   - Test B: cancel the shutdown token; server task finishes within the deadline; active worker count is 0.
2. **Module routes must require a session (T10 AC4).** `/api/v1/m/*` currently has no auth: the existing test calls `/api/v1/m/sample-hello/ping` without a cookie and expects 200. Fix the gate so no session → 401 `unauthorized` (checked before the 403 `module_disabled`). Update `server_tests.rs` and `p0_gate.rs` to log in first. Add a test: no cookie → 401 for both enabled and disabled modules. Also add the "list every router route and prove only health, login and setup-owner are public" test that T10 AC6 asked for (check whether it exists; if not, add it).
3. **Unix build is broken.** `dandori-process/src/executor.rs` uses `nix::...` inside `#[cfg(unix)]` but `nix` is declared nowhere, so the crate cannot compile on Linux/macOS (Windows never compiles that block). Also the child is not put in its own process group, so killing `-pid` would not work anyway. Fix without new libraries: spawn with `process_group(0)` on unix and kill the group with `kill -KILL -- -<pid>` via `tokio::process::Command` inside `dandori-process` (already allowed by `clippy.toml`). Then ask the owner (in the report) to run `rustup target add x86_64-unknown-linux-gnu` and `cargo check -p dandori-process --target x86_64-unknown-linux-gnu`. If you cannot run that, say so under "Chưa kiểm chứng".
4. **Login hardening (T10 item 3).**
   - Lockout by **source** as well as by account (spec says both). Use the peer address (`ConnectInfo`) or an explicit config; persist the counter.
   - Make the failed-attempt counter atomic (one `INSERT ... ON CONFLICT DO UPDATE SET failed_count = failed_count + 1` statement). The current read-then-write can undercount under concurrency.
   - Stop storing one row per arbitrary unknown username without bound (cap or key by source only for unknown names).
   - Remove `"token"` from the JSON body of `login` and `setup-owner` (the cookie is `HttpOnly`; returning the token to JS defeats that). If a test or the frontend needs it, change them to read the cookie and explain in the report.
   - `setup-owner`: do the "no user yet" check and the insert in one statement/transaction so two concurrent calls cannot create two owners. Add a concurrency test.
5. **Vault: bind ciphertext to its record.** Use AES-GCM associated data (record id) so swapping ciphertext between two records fails to decrypt. Add a test that swaps two rows and expects `DecryptionFailed`.
6. **Remove duplicate code.** `crates/mod-auth-login/src/auth.rs` is a copy of `crates/dandori-server/src/auth.rs`. Keep one place (the module owns the logic; the server calls it) and delete the copy.
7. **Weak tests found.** (Claude already tightened the timing test and added tests for the dummy hash, token hashing and the key-store clone bug.) Look for other assertions whose tolerance is larger than the thing they measure; fix them.

### Acceptance criteria

1. `pnpm run verify` green on a clean copy (paste raw output + duration).
2. Tests A and B (real server over TCP, graceful shutdown) green; paste output. Prove at least one of them goes RED when you break the code on purpose (then revert).
3. Unauthenticated request to a module route → 401 (test).
4. Source-based lockout test, atomic counter test, owner-race test, ciphertext-swap test: each has a RED proof.
5. No `nix` reference remains; report states exactly what was and was not compiled for unix.
6. Report `reports/T14B.md` in Vietnamese using the template; no "100%" wording; list everything unverified.
7. After DONE: set the row T14B to DONE; **do not** set "Cổng P0: đạt". Claude does that after re-review.
