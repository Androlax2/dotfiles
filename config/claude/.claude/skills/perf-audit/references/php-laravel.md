# PHP / Laravel Performance Checklist

Work top to bottom — ordered by how often each class of issue is the actual bottleneck.

## 1. Database access patterns (the usual culprit)

- **N+1 queries.** Grep for relationship access inside loops and Blade/`@foreach` templates (`$post->author->name` without eager loading). Check every `Model::all()` / `get()` whose results are iterated with relations. Fix: `with()` / `load()`; enforce with `Model::preventLazyLoading(!app()->isProduction())` in `AppServiceProvider` and flag if absent.
- **SELECT \*** on wide tables when few columns are used → `select([...])`; loading full models for counts → `withCount()` / `exists()` instead of `count(...->get())`.
- **Chunking absent** on large iterations: `all()`/`get()` over big tables in jobs/exports → `chunkById()`, `lazy()`, or `cursor()`. Memory findings live here too.
- **Queries in loops** (updates/inserts one-by-one) → bulk `upsert()`, `insert()`, or a single `whereIn` update.
- **Pagination**: `paginate()` runs a COUNT on every page — on large tables consider `simplePaginate()`/`cursorPaginate()`. OFFSET pagination deep into big tables is O(offset).

## 2. Caching

- What's cached vs recomputed per request? Config-driven candidates: expensive dashboard aggregates, settings, reference data, rendered fragments. Check `Cache::` usage, TTLs, and driver (`file`/`database` in prod is a smell at any scale where perf matters → Redis).
- `cache()->remember()` around queries that never change per-request but run every request.
- Invalidation: is anything cached with no invalidation path (stale bugs) or invalidated so aggressively the cache never hits?
- HTTP layer: `ETag`/`Cache-Control` on public endpoints, CDN in front of assets.

## 3. Work in the request path that belongs in a queue

- Mail, PDF generation, image processing, webhooks, third-party API calls made synchronously in controllers → `ShouldQueue`. Check `QUEUE_CONNECTION` isn't `sync` in production configs.
- External HTTP calls without timeouts (`Http::timeout()`) — a slow third party becomes your outage.

## 4. Framework & runtime configuration

- Production flags: `config:cache`, `route:cache`, `event:cache`, `view:cache` in the deploy script; `APP_DEBUG=false`. Composer installed with `--optimize-autoloader --no-dev`.
- **OPcache** enabled and sized (`opcache.validate_timestamps=0` on immutable deploys). If throughput is the complaint, note Octane (Swoole/FrankenPHP) as an M/L-effort option — don't lead with it.
- Session/cache drivers on `database` sharing the primary DB → contention; move to Redis.
- Middleware doing per-request work that could be cached or scoped to the routes that need it.

## 5. Serialization & responses

- API resources returning entire models (hidden cost: accessors firing, relations serialized) → explicit `JsonResource` fields.
- Missing gzip/brotli at the web server; huge JSON payloads that should paginate.
- Blade: heavy `@php` blocks and per-row queries in views (symptom of §1).

## 6. Jobs, crons & Horizon

- Jobs without `$tries`/`backoff` retrying hot loops; batch jobs loading everything into memory (see chunking); crons overlapping (`withoutOverlapping()` absent on long-running tasks).
- One default queue for everything → slow jobs starve fast ones; suggest priority queues.

## Measuring

- Query counts/timings: Telescope or Debugbar in local; `DB::listen()` + log in tests; `LOG_SLOW_QUERIES` style listener with a threshold (e.g. 100ms).
- `EXPLAIN` anything suspicious (see database.md).
- `php artisan about` for a quick config sanity read; Xdebug/Blackfire profiles only if the user has them set up.
