# 00. Lessons from the P2 independent review (read before P3, after 00_BAI_HOC_P1.md)

English, short. Every item is a defect that passed a green `pnpm run verify` in P2 and was found by Claude. Fixed in the tree; do not reintroduce.

## A. Secrets and network

1. **Secrets only through the vault, with the OS keyring.** Production code must use `dandori_vault::default_key_store()` (configured once by `init_server_state`: OS keyring unless `DANDORI_VAULT_BACKEND=memory`). Never `MemoryKeyStore` in production paths (key lost on restart, every stored secret becomes undecryptable). Store a `vault:<secret_id>` reference in the table, never the token. Deleting the row deletes the secret. Test: assert the DB and the `secret` ciphertext do not contain the raw value.
2. **Outbound HTTP (integrations, notifications, webhooks, anything new):** reuse the SSRF policy in `mod-integrations` (`ssrf.rs`, `http_client.rs`): https only, fixed provider hosts, resolve DNS and check every IP, pin the connection to the checked IPs (`resolve_to_addrs`), redirects only to the same host and scheme, block loopback/link-local/metadata/CGNAT/unspecified/multicast/IPv6 ULA and link-local/IPv4-mapped. Because modules may not depend on each other, P3-12 must first MOVE this code into a shared place (suggested: `dandori-module-api::safe_http`) and keep `mod-integrations` using it.
3. **Never detect "test mode" from the environment** (`CARGO_MANIFEST_DIR` is set by `cargo run`). Test-only switches are explicit: `dandori_module_api::set_allow_local_endpoints_for_tests(true)` in test setup.
4. **Provider token for a request must come from the workspace of the task/project, never "any active connection".** `DynamicGitHostProvider` fails closed if more than one workspace has connections; P2-07 (pr-merge) must pass the workspace through before using it.

## B. Who may do what

1. Roles: system admin (`SessionUser::is_system_admin`), workspace admin (`can_administer_workspace`), workspace member (`can_access_workspace`). Decisions and sensitive config need admin, not member: approving or rejecting, changing the approval policy, creating/deleting external connections, running any program on the host (`diff-preview` start), changing quota/limits, switching accounts.
2. Anything that lets a user choose a host-level capability (a command to run, `allow_private_network`, marking an account exhausted) is system-admin only.
3. For every route that takes a project id, task id, run id or any other record id: resolve the workspace from the stored record (`dandori_module_api::{project_workspace, task_workspace}` or a join), then check access. A login check alone is a defect.
4. Every new privileged endpoint needs three tests: outsider (not in the workspace) denied, plain member denied, admin allowed. Prove each with a mutation (remove the check, test must go RED).
5. Cross-record writes must check ownership both ways (importing issues into a project: the project must belong to the connection's workspace).
6. Expiry and state checks belong in the same atomic `UPDATE ... WHERE` as the decision (`status='pending' AND (expires_at IS NULL OR expires_at > now)`).

## C. Process rules

1. Do not loosen locked config. `cargo deny --offline check` was added in P1 and silently weakened the vulnerability check; it is reverted. Only ADD checks.
2. Never leave agent scratch files in the repo (`scratch/` is now git-ignored; use the OS temp dir).
3. A test that approves with a plain member contradicts the spec ("chủ dự án/admin duyệt"). Make test fixtures follow the spec roles, not what is convenient.
4. Test strings that look like shell invocations trip RS-SEC-001; build them at runtime (`["c","md"].concat()`).
5. Run the real `dandori.exe` for the touched area (P2 smoke: enable all modules, create an API-key account, check the DB for the raw key, restart, log in again). Include the transcript. Stop the server afterwards (it locks `target/debug/dandori.exe`).
6. A real OS-keyring credential `master-key.dandori-vault` is created on first vault use. Tests that touch the real keyring must clean it (`cmdkey /delete:"LegacyGeneric:target=master-key.dandori-vault"`); prefer in-memory key stores in tests.
