---
name: react-19-compiler
description: React 19 and React Compiler knowledge base grounded in the official React docs. Trigger when tasks mention React 19, React Compiler, `babel-plugin-react-compiler`, `eslint-plugin-react-hooks`, Actions, `useActionState`, `useOptimistic`, `useFormStatus`, `use`, `ref` as a prop, `forwardRef`, compiler directives, or manual memoization in compiler-enabled code.
---

# React 19 & Compiler

## Source Of Truth

- Read `references/official-sources.md` first.
- Treat `https://react.dev/` as canonical.
- As of 2026-03-09, react.dev documents React 19.2 and React Compiler v1.0. Distinguish stable docs from Canary-only pages before changing code.

## Fricon Context

- Frontend package: `crates/fricon-ui/frontend`.
- This repo already runs `react@19.2.x`, `react-dom@19.2.x`, and `babel-plugin-react-compiler@1.0.0`.
- The compiler is enabled in `crates/fricon-ui/frontend/vite.config.ts` through `@vitejs/plugin-react`.

## Scope

- React Compiler behavior, rollout concepts, and debugging.
- Compiler-powered ESLint remediation.
- React 19 form Actions and related hooks.
- Ref modernization (`ref` as a prop, `forwardRef`, `useImperativeHandle`).
- Manual memoization in compiler-enabled code.

## React 19 Defaults

1. Prefer Actions for mutations and forms.
Use `<form action={fn}>`, `useActionState`, `useFormStatus`, and `useOptimistic` before building custom pending or error plumbing.

2. Prefer `ref` as a prop in new function components.
Do not introduce `forwardRef` in new React 19-only code unless a compatibility requirement forces it.

3. Prefer modern React DOM APIs.
Use `createRoot` / `hydrateRoot`, DOM refs instead of `findDOMNode`, and default parameters instead of function `defaultProps`.

4. Treat `use` as a special case.
`use` may be called conditionally or in loops. Other Hooks still follow the normal Rules of Hooks.

5. Respect React 19 TypeScript changes.
`useRef` now requires an argument, ref callback cleanups cannot use implicit returns, and `ReactElement["props"]` defaults to `unknown`.

## Compiler Model

1. The compiler is an optimizing compiler, not a semantic escape hatch.
It assumes components and hooks follow the Rules of React. When code is impure or unsupported, the compiler skips it rather than preserving bad patterns.

2. The default mental model is `infer`.
Most apps should rely on compiler inference instead of annotating components manually.

3. Compiler diagnostics and ESLint are part of the same adoption story.
The React docs position compiler-backed lint rules as the practical way to surface purity, refs, unsupported syntax, and related issues incrementally.

4. Directives are override tools, not the normal API.
`"use memo"` is mainly for `annotation` mode or explicit opt-in. `"use no memo"` is primarily for isolating or excluding incompatible code.

5. Manual memoization still matters as historical code, not as the new default.
In compiler-enabled code, `useMemo`, `useCallback`, and `React.memo` are usually unnecessary for optimization, but existing uses should not be removed casually.

## Compiler Guidance

1. Start with project-level compiler config, not directives.
For most React 19 apps, the default compiler setup is enough.

2. Prefer compiler-led memoization for new code.
Avoid adding `useMemo`, `useCallback`, or `React.memo` by default in compiler-enabled code.

3. Treat `react-hooks/incompatible-library` as an explicit exception.
If `eslint-plugin-react-hooks` reports `react-hooks/incompatible-library`, React Compiler skips optimization for the affected component or hook. Treat that code as outside compiler assumptions and follow the library's own API and performance guidance instead. Prefer compiler-compatible alternative APIs when the library provides one.

4. Preserve existing manual memoization unless removal is verified.
Removing legacy memoization can change compiler output. Keep it unless tests or profiling show the rewrite is safe.

5. Use directives sparingly.
`"use memo"` is mainly for `annotation` mode or explicit overrides. `"use no memo"` is a temporary escape hatch for debugging or incompatible code.

6. Use lint-driven incremental adoption.
Upgrade `eslint-plugin-react-hooks`, then fix purity, immutability, refs, unsupported syntax, static component, and render/effect state issues over time. Compiler diagnostics skip only the unsafe components and hooks.

## High-Value Interpretations

1. In React 19 codebases with the compiler enabled, plain code is the baseline.
Extra memoization should be justified, not assumed.

2. Many "compiler bugs" are actually Rules of React violations.
Purity, ref access during render, mutation, and unsupported syntax are the first places to look.

3. React 19 form APIs reduce custom state plumbing.
`<form action>`, `useActionState`, `useFormStatus`, and `useOptimistic` replace many bespoke pending/error submission patterns.

4. `ref` as a prop changes the default shape of component APIs.
`forwardRef` becomes a compatibility boundary rather than the preferred starting point for new function components.

5. `use` is special, but only `use` is special.
Conditional or looped `use(...)` does not relax the ordinary Rules of Hooks for other hooks.

## Persistent Pitfalls

- Canary-only APIs and docs can be adjacent to stable docs; verify stability before relying on a page.
- Calling components as plain functions breaks the React model and confuses compiler assumptions.
- Render-time side effects, mutation, and ref access remain high-signal causes of compiler skips and lint findings.
- Removing effect dependencies or old manual memoization without proof can change behavior.
- `react-hooks/incompatible-library` means compiler memoization is unavailable for that boundary; do not mechanically apply or remove `useMemo` / `useCallback` based on compiler-era heuristics alone. Follow the library's own contract instead.
- Libraries that depend on React internals or outdated test infrastructure remain upgrade and compatibility risks.
