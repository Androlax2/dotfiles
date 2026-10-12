# Frontend / Next.js / React Performance Checklist

Ordered by typical real-world impact. Frame findings against Core Web Vitals where user-facing: LCP (loading), INP (interactivity), CLS (stability).

## 1. Bundle size & code splitting

- Run `next build` and read the route-level first-load JS. Anything red/large: find why. Standalone React: build with the bundler's analyzer (`vite-bundle-visualizer`, `webpack-bundle-analyzer`).
- Heavyweight deps in the client bundle: moment (→ date-fns/dayjs), lodash full import (→ per-method or native), chart/editor/map libraries loaded eagerly on routes that rarely use them → `next/dynamic` / `React.lazy` with a loading state.
- Barrel-file imports pulling whole libraries; check `optimizePackageImports` for icon/component kits.
- Polyfills and duplicate dependencies (two React versions, two date libs) in the lockfile.
- Client components that could be Server Components (App Router): grep `"use client"` — each one pulls its import subtree into the bundle. Push the directive down to the leaf that actually needs interactivity.

## 2. Data fetching & rendering strategy

- Waterfalls: sequential `await`s for independent data → `Promise.all`; client `useEffect`-fetch chains for data the server could have sent (RSC / route loaders / getServerSideProps).
- Wrong rendering mode: fully dynamic SSR for pages that could be static/ISR (`revalidate`); or SSG rebuilding thousands of rarely-visited pages.
- Missing `fetch` caching/deduplication config in App Router; duplicated client fetches → SWR/React Query with sane staleTime.
- Overfetching: endpoints returning far more than the view renders (coordinate with backend findings).

## 3. Assets

- Images: raw `<img>` with full-size originals → `next/image` (or explicit `srcset` + modern formats). Missing `width/height` (CLS), missing `priority` on the LCP image, lazy-loading the hero (hurts LCP) or eager-loading below-the-fold.
- Fonts: render-blocking font CSS, no `font-display: swap` → `next/font`/self-hosting; icon fonts → SVG.
- Video/GIF: animated GIFs → muted looping video.

## 4. Interactivity (INP)

- Expensive work in render or event handlers on large lists: missing virtualization (`react-virtual`/`react-window`) for 500+ row lists; heavy computation → `useMemo` or move server-side; synchronous state updates triggering wide re-renders (context value recreated every render, missing `memo` on expensive pure subtrees — flag only with evidence of actual re-render cost, not ritually).
- Third-party scripts (analytics, chat widgets) loaded `beforeInteractive` → `next/script` with `lazyOnload`/`afterInteractive`; tag-manager bloat is often the single biggest INP cost.
- Long hydration: giant client components above the fold; suspense boundaries absent so the whole page waits on the slowest data.

## 5. Network & delivery

- No CDN/edge caching for static assets; missing `Cache-Control: immutable` on hashed assets.
- Uncompressed responses (check brotli/gzip); API and page on different origins forcing extra DNS/TLS + CORS preflights (`OPTIONS` storms on every request → cache preflight, same-origin proxy).

## Measuring

- `next build` output + `@next/bundle-analyzer` for bundles.
- Lighthouse (in CI if possible) for LCP/INP/CLS lab numbers; note lab ≠ field, real-user data (CrUX, Vercel Analytics) wins when available.
- React DevTools Profiler for re-render hotspots — only when the complaint is interaction jank.
