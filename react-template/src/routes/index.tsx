import { createFileRoute, Link } from "@tanstack/react-router";
import { useCounterStore } from "@/lib/useCounterStore";

export const Route = createFileRoute("/")({
  component: HomeComponent,
});

function HomeComponent() {
  const count = useCounterStore((state) => state.count);
  const increment = useCounterStore((state) => state.increment);
  const decrement = useCounterStore((state) => state.decrement);
  const reset = useCounterStore((state) => state.reset);
  const addBy = useCounterStore((state) => state.addBy);

  return (
    <div className="space-y-8 py-8">
      <div className="text-center">
        <h1 className="text-foreground text-4xl font-bold">TanStack Demo</h1>
        <p className="text-muted-foreground mt-2 text-lg">
          Router + Query + Table + Zustand
        </p>
      </div>

      <div className="mx-auto grid max-w-3xl gap-4 md:grid-cols-3">
        <div className="bg-card rounded-lg border p-6">
          <h2 className="text-xl font-semibold">Router</h2>
          <p className="text-muted-foreground mt-2 text-sm">
            File-based routing with type-safe navigation
          </p>
        </div>
        <div className="bg-card rounded-lg border p-6">
          <h2 className="text-xl font-semibold">Query</h2>
          <p className="text-muted-foreground mt-2 text-sm">
            Powerful async state management
          </p>
        </div>
        <div className="bg-card rounded-lg border p-6">
          <h2 className="text-xl font-semibold">Table</h2>
          <p className="text-muted-foreground mt-2 text-sm">
            Headless UI for building tables
          </p>
        </div>
      </div>

      <div className="bg-card mx-auto max-w-3xl rounded-lg border p-6">
        <div className="flex flex-wrap items-center justify-between gap-4">
          <div>
            <h2 className="text-xl font-semibold">Zustand Counter</h2>
            <p className="text-muted-foreground mt-1 text-sm">
              全局状态示例：不同组件可共享这个计数值
            </p>
          </div>
          <div className="text-3xl font-semibold tabular-nums">{count}</div>
        </div>

        <div className="mt-4 flex flex-wrap gap-2">
          <button
            type="button"
            onClick={decrement}
            className="bg-secondary text-secondary-foreground hover:bg-secondary/80 inline-flex items-center rounded-md px-3 py-2 text-sm font-medium"
          >
            -1
          </button>
          <button
            type="button"
            onClick={increment}
            className="bg-primary text-primary-foreground hover:bg-primary/90 inline-flex items-center rounded-md px-3 py-2 text-sm font-medium"
          >
            +1
          </button>
          <button
            type="button"
            onClick={() => addBy(5)}
            className="bg-accent text-accent-foreground hover:bg-accent/80 inline-flex items-center rounded-md px-3 py-2 text-sm font-medium"
          >
            +5
          </button>
          <button
            type="button"
            onClick={reset}
            className="bg-muted text-muted-foreground hover:bg-muted/80 inline-flex items-center rounded-md px-3 py-2 text-sm font-medium"
          >
            Reset
          </button>
        </div>
      </div>

      <div className="text-center">
        <Link
          to="/users"
          className="bg-primary text-primary-foreground hover:bg-primary/90 inline-flex items-center rounded-md px-4 py-2 text-sm font-medium"
        >
          View Users Demo →
        </Link>
      </div>
    </div>
  );
}
