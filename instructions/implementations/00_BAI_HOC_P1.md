# 00. Lessons from the P0 and P1 independent reviews (read before P2)

Written in English to save tokens. Source: `reports/REVIEW_P0.md`, `reports/REVIEW_P1.md`. Every item below is a defect that passed a green `pnpm run verify` and was found by Claude by reading code, probing, or running the real binary. Do not repeat them.

## A. Behaviour rules

1. **A green test is not proof.** For every security or lifecycle behaviour, break the code on purpose, show the test going RED, revert. Tests that assert nothing real are defects (a "workers stop on shutdown" test with zero workers; a timing tolerance larger than the effect).
2. **Run the real binary.** At the end of each task run `target/debug/dandori.exe` (env `DANDORI_DATA_DIR` temp dir, `DANDORI_PORT` free port) and probe it over HTTP. Put the transcript in the report. The P1 smoke script pattern: setup-owner, apply `solo-light` modules, create workspace/project/account/task, dispatch, poll, cancel. `DANDORI_ENABLE_FAKE_PROVIDER=1` enables the fake agent; it must stay off otherwise (fake-agent scenarios can write arbitrary files).
3. **Authorization is per module.** The registry gate only checks "valid session". Every module route that reads or writes workspace data MUST call `dandori_module_api::{session_user, can_access_workspace, accessible_workspace_ids}`. Derive the workspace from the stored record (prompt, project, task), never trust a `workspace_id` sent by the client. Add a test with a second, non-member user (see `p1_07`, `p1_08`, `p1_06`).
4. **One source of truth.** Do not copy helpers across crates (session token hashing is `dandori_db::hash_session_token`; secret detection is `dandori_module_api::contains_secret_like`; start-run error mapping is `routes::runs::map_start_run_error`).
5. **Never pass user text to a CLI or git as an option.** Validate refs with `dandori_git::validate_ref_name`; put `--` before free text (the prompt already does).
6. **Cancel must kill the agent.** Dropping the event receiver kills the process tree (`dandori-process`, `next_event` in adapters). Keep that property for any new runner.
7. **Atomic state changes.** Claim with a conditional `UPDATE ... WHERE status NOT IN (...)` and check `rows_affected`; get every resource that can fail (adapter) before changing state; roll back on failure.
8. **Do not guess external facts.** Read the real `--help` or output and paste verbatim (a paraphrase is not verbatim). If you cannot verify, mark UNVERIFIED in the report and in code. Codex adapter is disabled (`ARGS_VERIFIED = false`) for this reason.
9. Windows is the only platform available. Anything under `#[cfg(unix)]` is UNVERIFIED; say so.
10. Never run real CLIs, real accounts or network in tests (a tiny real Claude run costs about 0.25 USD of quota). Use fake-agent and recorded fixtures.
11. Reports: Vietnamese, template in `00_MAU_BAO_CAO.md`, raw command output, no "100%/perfect". A criterion not met is not "Qua".

## B. Real Claude CLI facts (verified 2026-11-01 on Claude Code 2.1.300)

- Command used by the adapter: `claude -p --output-format stream-json --verbose --dangerously-skip-permissions -- "<prompt>"`. `--verbose` is mandatory with stream-json; `--` is needed for prompts starting with `-`.
- `claude auth status` prints JSON with `loggedIn` (exit 1 when not logged in) and honours the isolated HOME. Account becomes `ready` through `POST .../accounts/{id}/check`. Providers without a verified status command (`agy`, `codex`) become `ready` only with body `{"confirm_logged_in": true}`.
- On Windows (npm install) `claude` is `claude.cmd`; `dandori_process::resolve_executable` maps it to the native `claude.exe`.
- Each stream line is JSON with `type`: `system` (init, commands_changed), `assistant` (content blocks `text` and `tool_use`), `user` (`tool_result`), `rate_limit_event`, `result`. Recorded sample: `crates/dandori-agent-adapters/tests/fixtures/claude_stream_real.jsonl`. The parser currently DROPS `rate_limit_event` and `result` success lines.
- `rate_limit_event.rate_limit_info`: `status` ("allowed"), `resetsAt` (unix seconds), `rateLimitType` ("five_hour"), `overageStatus`, `isUsingOverage`.
- `result`: `total_cost_usd`, `usage` (`input_tokens`, `cache_creation_input_tokens`, `cache_read_input_tokens`, `output_tokens`), `num_turns`, `duration_ms`, `is_error`, `result`.
- P2-09 (`quota-tracking`) and P2-12 (`cost-tracking`) therefore have a real `reported` source for Claude. Add NEW `AgentEvent` variants additively (for example `RateLimit` and `Usage`), emit them from the Claude parser using the fixture, store them in `run_event`, and keep `agy`/`codex`/`fake` as "no data" (never estimate tokens). The 5-hour reset time is a real value, not a guess.
- Cache facts (official docs): read 0.1x (0.05x on Opus 5.5 and Sonnet 5.5), write 1.25x for 5 min, default lifetime 5 minutes, prefix match. Do not implement cache tricks in P2; see `docs/y-tuong/bo-nho-va-tim-ma-nguon.md` (backlog B-01..B-10).

## C. Known open items you must not "fix" silently

`agy` never run for real; unix code never compiled; UI never opened in a browser; concurrency limit can be exceeded by a burst (B-06). Report them as UNVERIFIED if your task touches them.
