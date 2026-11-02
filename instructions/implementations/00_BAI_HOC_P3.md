# 00. Lessons from the P3 independent review (read before P4, after 00_BAI_HOC_P1.md and 00_BAI_HOC_P2.md)

English, short. Every item is a defect that passed a green `pnpm run verify` in P3 and was found by Claude. Fixed in the tree; do not reintroduce.

## A. Approval, trust and prompts

1. **Anything that gates what an agent will read must be approved by a workspace admin, not any member.** Memory approve/reject (single and batch), skill approval and instruction changes: `can_administer_workspace`. A plain member's entry is created `pending`. A test that lets a member approve contradicts the spec; write fixtures with the spec roles.
2. **Prompt contributors must filter by the workspace of the task's project.** `WHERE project_id = ? OR project_id IS NULL` without a workspace filter leaks every workspace's "global" rows into every prompt. Any new `PromptContributor` needs a cross-workspace test.
3. **Stored text injected into a prompt is untrusted.** Flatten it to one line and neutralise block markers (`<!--`, `-->`) so it cannot close the fence you put around it.
4. **A project id sent by the client must belong to the workspace in the URL.** Check it on every create/propose.

## B. Audit log

1. **The actor is never client-supplied.** Take it from the session. Same for `source`.
2. **An audit store nobody writes to is not an audit log.** Verify (by grep for `INSERT INTO <table>` outside the module's own routes, and by a test that triggers the real event) that sensitive events actually produce records. Events used for this need a workspace in their `EventScope`.
3. **Immutable data must be redacted before it is stored,** including truncated private keys (no END line, PGP blocks). Redaction runs before hashing.
4. **Serialise chain appends** (one lock) or two writers will produce the same `sequence`.

## C. Files, archives, symlinks

1. **Never read, write or recurse through a symlink or junction that comes from a repository or an upload.** Use `symlink_metadata` / `DirEntry::file_type` (not `exists()`, `is_dir()`, `is_file()`, which follow links). Applies to `WorkdirDecorator`, skill folders, `.cron/jobs`.
2. **Decompression limits use the real output, not the size declared in the header.** Track a running total of bytes actually produced and stop at the cap.
3. **A path a user types that points at the server's disk is a host capability:** system admin only.
4. **Put a depth and size cap on every recursive or file read from untrusted input.**

## D. Network policy and configuration

1. **Matching a domain with `ends_with` is a bug.** Use a dot boundary: `*.example.com` matches `a.example.com`, never `evilexample.com`. Reject entries like `*`, `*.com`. One shared function (`dandori_module_api::origin_host_matches`) is the only matcher.
2. **Configuration written by an admin of one workspace must not widen a system-wide security check** unless it is validated and bounded.
3. **Production code never passes `allow_private = true`.** Only the explicit test switch relaxes the SSRF rules.
4. **Bound every number a client sends** (`ttl_hours`): `Duration::hours(i64::MAX)` panics.
5. **Mentions in outgoing chat messages** (`@everyone`) must be disabled when the text can contain user input.

## E. Process

1. A feature is not DONE if its main purpose is untested end to end. Test the real trigger (toggle a module, see an audit record), not only the endpoint that simulates it.
2. When a fix changes a first-record assumption in an older test (`sequence == 1`), update the test and say why; do not weaken the assertion.
3. Keep functions at or below 50 lines (clippy `too_many_lines`); extract helpers instead of adding `#[expect]`.
4. Symlink tests may be skipped on Windows without privilege. Say so in the report and verify once by hand (a junction via `mklink /J` works without admin).
