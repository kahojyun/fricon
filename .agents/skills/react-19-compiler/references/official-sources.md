# Official Sources

Use only the pages relevant to the task. React docs track the latest major version, not every patch release.

## Version And Release Anchors

- `https://react.dev/versions`
Confirms the latest documented major version. As of 2026-03-09, react.dev shows React 19.2.

- `https://react.dev/blog/2024/12/05/react-19`
Stable React 19 release overview: Actions, `useActionState`, `useOptimistic`, `<form action>`, `useFormStatus`, `ref` as a prop, Suspense, and static API changes.

## Compiler

- `https://react.dev/blog/2025/10/07/react-compiler-1`
React Compiler v1.0 stable announcement, lint integration, incremental adoption, and guidance for `useMemo` / `useCallback` / `React.memo`.

- `https://react.dev/reference/react-compiler/configuration`
Default config, `target`, `panicThreshold`, `gating`, and incremental adoption patterns.

- `https://react.dev/reference/react-compiler/directives`
When to use `"use memo"` and `"use no memo"`.

- `https://react.dev/reference/react-compiler/directives/use-memo`
Read when the task uses `annotation` mode or needs an explicit opt-in.

- `https://react.dev/reference/react-compiler/directives/use-no-memo`
Read when isolating compiler issues or handling incompatible code.

## React 19 APIs

- `https://react.dev/reference/react/useActionState`
Form actions with state and pending handling.

- `https://react.dev/reference/react-dom/components/form`
`<form action>` behavior, uncontrolled reset semantics, server-function integration, and examples.

- `https://react.dev/reference/react-dom/hooks/useFormStatus`
Form pending state and submission metadata for nested components.

- `https://react.dev/reference/react/useOptimistic`
Optimistic UI patterns for mutations.

- `https://react.dev/reference/react/use`
Read when the task suspends on promises or context in render.

- `https://react.dev/reference/react/forwardRef`
React 19 deprecation direction for `forwardRef`.

- `https://react.dev/reference/react/useImperativeHandle`
Imperative handle guidance; note that `ref` is a prop in React 19.

## Rules And Lints

- `https://react.dev/reference/eslint-plugin-react-hooks`
Compiler-powered lint overview and adoption model.

- `https://react.dev/reference/rules/components-and-hooks-must-be-pure`
Purity, idempotency, render-side effects, and mutation rules.

- `https://react.dev/reference/eslint-plugin-react-hooks/lints/refs`
Ref safety rules, especially avoiding `ref.current` reads or writes during render.

- `https://react.dev/reference/eslint-plugin-react-hooks/lints/preserve-manual-memoization`
Read before removing existing `useMemo`, `useCallback`, or `React.memo`.

- `https://react.dev/reference/eslint-plugin-react-hooks/lints/unsupported-syntax`
Read when the compiler skips code because syntax or patterns are unsupported.
