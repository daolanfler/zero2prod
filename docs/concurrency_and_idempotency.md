# Concurrency, Idempotency, and Delivery Backoff

This document summarizes how the codebase handles duplicate requests (idempotency),
concurrency coordination in Postgres, and transient failures in the newsletter delivery worker.

## Idempotency (POST /admin/newsletters)

- User provides an `idempotency_key` (hidden field in the form). The key is validated (non-empty, < 50 chars).
- `try_processing(pool, key, user_id)` starts a transaction and executes:
  - `INSERT INTO idempotency (user_id, idempotency_key, created_at) VALUES (...) ON CONFLICT DO NOTHING`
  - If `rows_affected > 0`: this is the first request → return `StartProcessing(transaction)`.
  - Else: another request already “claimed” the key → load the saved response and return it (`ReturnSavedResponse`).
- First request path:
  1) Execute business logic inside the same transaction (insert newsletter issue, enqueue tasks).
  2) Build the HTTP response (303 to `/admin/newsletters`).
  3) `save_response(transaction, key, user_id, response)` serializes status, headers, and body into the `idempotency` table and commits.
- Subsequent/concurrent requests:
  - Wait on the unique-key insert until the first commits (see Postgres behavior below), then rehydrate the saved response and return it.

### Storage

- Table `idempotency` PK: `(user_id, idempotency_key)` → idempotency is scoped per user.
- Columns include `response_status_code`, `response_headers` (array of `header_pair`), `response_body`, and `created_at`.
- Migrations relax NOT NULL on response columns so the “claim” insert can happen before the “save”.

### Guarantees and Limits

- Guarantees: No duplicate side effects for the same user + key; later callers receive the exact same HTTP response (status, headers, body).
- Implemented for newsletter publishing. No automatic cleanup of idempotency records.

## Postgres Concurrency & Isolation

- Concurrency coordination relies on a unique constraint (the PK) and `INSERT ... ON CONFLICT DO NOTHING`.
- With concurrent inserts of the same `(user_id, idempotency_key)`, the second transaction blocks until the first commits or aborts.
- Isolation assumptions:
  - Default `READ COMMITTED` is expected. After the first commits and saves the response, the second transaction can see it.
  - Stricter levels (`REPEATABLE READ`/`SERIALIZABLE`) can cause stale snapshots or serialization failures unless you add retry logic. The code comments call this out.

## Newsletter Delivery Worker (Transient Failures & Backoff)

- Loop (`worker_loop`):
  - `EmptyQueue` → sleep 10s (avoid hammering DB).
  - `Err(_)` (unexpected worker/DB error) → sleep 1s (simple backoff).
  - `TaskCompleted` → immediately fetch next task.
- Task execution (`try_execute_task`):
  - Claim 1 job with `SELECT ... FOR UPDATE SKIP LOCKED LIMIT 1` to avoid contention.
  - Load issue and attempt `send_email`.
  - On email send failure: log error and still delete the task (no retry).

### Implications

- Backoff applies to empty-queue polling and unexpected worker errors, not to email send failures.
- Email delivery is “at most once”: failed deliveries are not retried under the current design.

## Possible Improvements

- Add retry semantics for failed deliveries:
  - Track `attempts`, `next_attempt_at`, and error cause per task.
  - Use exponential backoff with jitter on failures; only delete after success or exceeding max attempts.
  - Dequeue using `WHERE next_attempt_at <= now()`.
- Add cleanup/TTL policy for idempotency rows.

## Pointers

- Idempotency: `src/idempotency/{mod.rs,key.rs,persistence.rs}` and usage in `src/routes/admin/newsletters/post.rs`.
- Worker & backoff: `src/issue_delivery_worker.rs`.
- Postgres schema: `migrations/*` (idempotency table, newsletter issues, delivery queue).

