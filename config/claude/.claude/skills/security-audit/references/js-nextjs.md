# Next.js / React / Node Security Checklist

## Grep targets

| Pattern | Risk |
|---|---|
| `dangerouslySetInnerHTML` | XSS unless sanitized with DOMPurify (check the sanitizer runs server-side too for SSR'd content) |
| `eval(`, `new Function(`, `vm.` | Code injection |
| `child_process` (`exec` especially) | Command injection — `execFile`/`spawn` with array args |
| `NEXT_PUBLIC_` prefixed vars | Anything so named ships to the browser — no secrets, API keys with server-side privileges, or signing material |
| `res.redirect(`/`redirect(` with request-derived URLs | Open redirect — allowlist paths |
| `JSON.parse` of request bodies into ORM calls | Operator injection (Mongo/Prisma raw), mass assignment via object spread into `update` |
| `href={userValue}` | `javascript:` URL XSS — validate scheme |
| `process.env` read in client components | Silent `undefined` or accidental exposure via bundling |

## Next.js-specific

- **Server/client boundary**: secrets only in server code (Route Handlers, Server Components, server actions). Check no server-only module is imported from a `"use client"` tree (`server-only` package enforces this — suggest it). Props passed from Server to Client Components are serialized into the HTML — don't pass whole DB records containing fields the user shouldn't see.
- **Server Actions** are public HTTP endpoints: they need the same auth/authz checks as API routes — verify session INSIDE the action, never trust that "only my form calls this". Validate arguments (zod) — the client can send anything.
- **Route Handlers / API routes**: per-route auth (no assumption that middleware covered it — verify the matcher actually includes the path); middleware-based auth alone has had bypass CVEs (e.g. the `x-middleware-subrequest` bypass, CVE-2025-29927) → do auth checks in the handler/layout too, and check the Next.js version against known advisories.
- **Middleware matcher gaps**: paths excluded by regex (static, images) that also match app routes.
- **SSRF surface**: `next/image` remote patterns overly broad (`**` hostname), any fetch of user-supplied URLs in Route Handlers or OG-image generators.
- **Draft/preview mode** endpoints gated by secret; ISR revalidate endpoint requires a token.
- Error handling: `NODE_ENV=production` so stack traces/digest-only errors; no `serverRuntimeConfig` leaked through props.

## Auth in the JS ecosystem

- Session strategy: JWT-in-localStorage is XSS-stealable — prefer httpOnly cookies; if cookies, CSRF story required (SameSite + origin checks on mutations).
- NextAuth/Auth.js/Lucia/etc.: callback URLs restricted, `trustHost` not blindly true, OAuth `state`/PKCE handled by the lib (don't hand-roll), account-linking by unverified email disabled.
- Password flows (if custom): same rules as core checklist — bcrypt/argon2, rate limiting, no enumeration.

## Node/runtime

- Prototype pollution: deep-merge of request objects (lodash `merge` on user input), `__proto__` keys — validate/clone with null-prototype or use safe merges.
- ReDoS: user input against catastrophic regexes in hot paths.
- `Content-Security-Policy` for apps rendering user content (Hardening unless XSS was found — then it's part of the fix); `X-Frame-Options`/`frame-ancestors` for anything with auth'd UI.

## Tooling

`npm audit` / `pnpm audit` (triage: only runtime deps' criticals matter urgently; dev-dep noise goes to one line). Check `next` version against recent security advisories. Lockfile committed; no install scripts from unvetted packages if the org cares (note only).
