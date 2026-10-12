# PHP / Laravel Security Checklist

The framework is safe by default; audit the deviations. Grep targets first, then config.

## Grep targets (escape hatches)

| Pattern | Risk |
|---|---|
| `DB::raw`, `whereRaw`, `selectRaw`, `orderByRaw`, `havingRaw` | SQLi if any argument interpolates request data — bindings array must carry the values; column/direction from input must be allowlisted |
| `{!! !!}` in Blade | XSS unless the value is provably safe (rendered markdown → still needs sanitizing) |
| `unserialize(` | Object injection — use `json_decode`; if unavoidable, `allowed_classes: false` |
| `exec`, `shell_exec`, `system`, `proc_open`, `passthru`, backticks | Command injection — `Process` with array args, `escapeshellarg` |
| `$request->all()` into `create`/`update`/`fill` | Mass assignment — check `$fillable` excludes privilege/ownership columns (`role`, `is_admin`, `user_id`); prefer `$request->validated()` |
| `->withoutGlobalScopes`, `withoutMiddleware` | Scope/auth bypass — justify each |
| `env(` outside config files | Breaks with `config:cache` and hints at ad-hoc secret handling |
| `md5(`, `sha1(` on anything auth-related | Weak hashing — `Hash::make` / `hash_equals` for comparisons |
| `Storage::url` / `asset(` on private files | Unauthenticated access to user data |
| `@csrf` absent in forms / entries in `VerifyCsrfToken::$except` | CSRF |

## Authorization patterns

- Route model binding does NOT authorize: `Route::get('/invoices/{invoice}', ...)` resolves anyone's invoice — require a Policy check (`$this->authorize('view', $invoice)`, `can:` middleware) or scoped binding (`{invoice}` bound through the relationship / `->scopeBindings()`).
- Policies exist for every model exposed via routes; `Gate::before` admin overrides reviewed.
- API routes (`routes/api.php`) mirror web authorization — the classic hole is a policied web route with a policy-free API twin.
- Filament/Nova/admin panels: panel access gated (`canAccessPanel`), resources respect policies, admin routes not just "hidden".

## Config & environment

- `APP_DEBUG=false` in production (debug pages leak env, and Ignition-era CVEs made debug mode RCE); `APP_ENV=production`; `APP_KEY` set, unique, never committed or rotated carelessly (invalidates encrypted data).
- `.env` outside webroot / server configured to never serve dotfiles; `.env*` in `.gitignore` AND absent from git history.
- Session config: `secure`, `http_only`, `same_site` at least `lax`; session driver not `cookie` if the payload holds anything sensitive.
- `TrustProxies`/`TrustHosts` configured when behind a proxy (host-header injection affects password-reset links).
- CORS (`config/cors.php`): no `*` origin together with credentials; allowed origins explicit.
- Signed URLs used for email actions (`URL::signedRoute`), and signature actually validated (`signed` middleware).
- Rate limiting on login, password reset, and public write endpoints (`ThrottleRequests` — verify limiter definitions, not just middleware presence).

## Queues, jobs, webhooks

- Webhook endpoints verify signatures (Stripe/etc.) before processing; excluded from CSRF for a documented reason only.
- Job payloads treated as data — no user-controlled class names/methods dispatched.

## Tooling

`composer audit` for known CVEs (core + packages — Laravel point releases patch security; flag EOL major versions). If installed: Psalm/PHPStan with security rules, Enlightn. Suggest, don't install unasked.
