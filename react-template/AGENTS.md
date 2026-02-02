# Template for React application

## Project React Setup Summary

This is a modern React 19 project with the following setup:

- **Build Tool**: Vite 7
- **Language**: TypeScript 5.9
- **Styling**: Tailwind CSS 4 with CSS variables
- **UI Components**: shadcn/ui (base-mira style) with Base UI and Lucide icons
- **Routing**: TanStack Router (file-based)
- **Data Fetching**: TanStack Query (React Query)
- **Tables**: TanStack Table
- **Linting**: ESLint 9 with React hooks, React DOM, and Prettier integration
- **Formatting**: Prettier with Tailwind CSS plugin
- **React Compiler**: Babel plugin for React Compiler enabled

### Key Commands

- `pnpm dev` - Start development server
- `pnpm build` - Build for production and generate route tree
- `pnpm lint` - Run ESLint
- `pnpm format --check` - Check formatting with Prettier
- `pnpm fix` - Fix linting and formatting issues
- `pnpm preview` - Preview production build
- `pnpm type-check` - Run TypeScript type checking
- `pnpm test --run` - Run unit tests once (avoid watch mode in agent runs)

### Path Aliases

- `@/*` maps to `./src/*`

### File-Based Routing Pitfalls

When working with TanStack Router's file-based routing, be aware of:

1. **File naming conventions are strict** - Route files must follow exact naming patterns (`route.tsx`, `index.tsx`, `$param.tsx` for dynamic segments)
2. **Route generation requires dev server restart** - Adding new route files may require restarting the dev server for the route tree to regenerate
3. **Avoid circular imports** - Route files should not import from each other; use shared modules in `lib/` or `utils/`
4. **Layout nesting can be confusing** - Understand the difference between `_layout.tsx` (pathless layout) and regular nested routes
5. **Dynamic segments naming** - Files prefixed with `$` become dynamic route parameters (e.g., `$userId.tsx` → `/users/:userId`)
6. **Catch-all routes** - Use `$.tsx` for catch-all routes, but be careful with route priority
7. **Type generation** - The `routeTree.gen.ts` file is auto-generated; never edit it manually
8. **Colocation limitations** - Non-route files in the routes directory may interfere with route generation; use `.` prefix to exclude them (e.g., `.components/`)
9. **Tests in routes need `-` prefix** - Test files under `src/routes/` are allowed only when the filename or folder is prefixed with `-` so the generator skips them

### Testing Setup

- **Framework**: Vitest + React Testing Library + jsdom
- **Config**: `vitest.config.ts` (merged with Vite config)
- **TypeScript**: `tsconfig.test.json` for test types and globals
- **Test locations**: Prefer colocated tests next to source files; route-related tests can live under `src/routes/` with a leading `-` prefix to avoid route generation
- **Agent runs**: Always use `pnpm test --run` to avoid hanging watch mode

## Agent Instructions for Autonomous Work

### Workflow Guidelines

1. **Always verify before modifying** - Run `pnpm type-check` and `pnpm lint` before and after making changes to ensure no regressions
2. **Auto-fix issues** - Use `pnpm fix` to automatically resolve linting and formatting issues after code changes
3. **Test builds** - Run `pnpm build` to verify production builds work correctly after significant changes
4. **Run tests once** - Use `pnpm test --run` (never watch) during agent execution

### Code Generation Standards

1. **Use TypeScript strictly** - All new files must be `.ts` or `.tsx` with proper type annotations
2. **Follow existing patterns** - Match the code style and patterns already established in the codebase
3. **Use path aliases** - Always use `@/` imports instead of relative paths for `src/` files
4. **Component structure** - Place reusable components in `src/components/`, UI primitives in `src/components/ui/`
5. **Utilities** - Shared utilities go in `src/lib/`, hooks in `src/hooks/`
6. **Prefer shadcn components** - When a component is missing, first check if shadcn/ui has an existing component before creating custom ones. Use `pnpm shadcn add <component-name>` to add available components.

### Autonomous Task Execution

1. **Read before writing** - Always examine existing code structure before creating new files
2. **Incremental changes** - Make small, focused changes rather than large rewrites
3. **Validate changes** - After modifications, run the appropriate check commands
4. **Handle errors** - If a command fails, analyze the error output and fix issues before proceeding
5. **Document decisions** - Add comments for non-obvious code decisions

### Common Tasks

- **Adding a new route**: Create file in `src/routes/` following naming conventions, then restart dev server if needed
- **Adding a component**: First check shadcn/ui availability, then create in appropriate directory, export from index if pattern exists
- **Adding a shadcn component**: Use `pnpm shadcn add <component-name>` - always prefer this over custom implementations
- **Modifying styles**: Use Tailwind classes, check `src/index.css` for CSS variables
