# Database Performance Checklist

Applies to MySQL/MariaDB and PostgreSQL; note engine-specific points inline. Read alongside the backend reference — most DB findings surface as ORM patterns.

## 1. Indexes

- **Missing indexes on filter/join columns.** Cross-check every frequent `WHERE`, `JOIN ... ON`, and `ORDER BY` column against the schema/migrations. Foreign keys without indexes (Postgres does NOT auto-index FKs; MySQL does).
- **Composite index order**: equality columns first, then range/sort. An index on `(status)` doesn't serve `WHERE status = ? AND created_at > ? ORDER BY created_at` — `(status, created_at)` does.
- **Unused/duplicate indexes** slowing writes: `(a)` alongside `(a, b)` is usually redundant.
- Functions on indexed columns in WHERE (`DATE(created_at) = ?`, `LOWER(email) = ?`) defeat the index → range predicates or functional/expression indexes.
- `LIKE '%term%'` can't use a btree → full-text index (or dedicated search) if it's a hot path.

## 2. Query plans

`EXPLAIN` (MySQL: `EXPLAIN ANALYZE` on 8.0+; Postgres: `EXPLAIN (ANALYZE, BUFFERS)`) every query flagged in the sweep. Red flags:

- Sequential/full table scan on large tables in hot paths
- `filesort` / external sort on big result sets → covering index including the ORDER BY
- Row estimate wildly off actual (stale statistics → `ANALYZE`)
- Nested loop over large sets where a hash join was expected (Postgres)

## 3. Query shape

- `OFFSET` pagination deep into large tables → keyset/cursor pagination.
- `SELECT *` feeding large sorts or network transfer; huge `IN (...)` lists → joins or temp tables.
- Correlated subqueries executed per-row → JOIN or lateral; `DISTINCT` papering over a join fan-out.
- Aggregates computed per-request that belong in a cached/materialized rollup (or counter cache columns for hot counts).

## 4. Writes, locks & transactions

- Long transactions holding locks (external API calls inside a transaction is the classic) → shrink transaction scope.
- Row-by-row inserts/updates in loops → bulk operations; missing `ON CONFLICT`/`ON DUPLICATE KEY` for upserts.
- Hot-row contention (single counter row updated by every request) → sharded counters, Redis, or queue-batched updates.
- Postgres: mass UPDATEs churning dead tuples → check autovacuum keeps up; MySQL: gap-lock surprises under high-concurrency inserts.

## 5. Connections & config (flag, don't tune blindly)

- No connection pooling with many app processes (Postgres especially → pgbouncer); pool exhaustion under load.
- Defaults never touched on a dedicated DB server: `innodb_buffer_pool_size` / `shared_buffers` sized to a fraction of RAM. One-line finding pointing at the parameter, not a full tuning session.
- Slow query log enabled with a threshold (e.g. 100–200ms) — if it's off, turning it on is itself a quick win.

## Evidence to include per finding

The query (parameterized), where it's generated (file:line), the EXPLAIN summary in one line, table row count if known, and the proposed index/rewrite as runnable SQL or a migration.
