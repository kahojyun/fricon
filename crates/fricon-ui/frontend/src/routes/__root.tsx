import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { ReactQueryDevtools } from "@tanstack/react-query-devtools";
import { createRootRoute, Link, Outlet } from "@tanstack/react-router";
import { TanStackRouterDevtools } from "@tanstack/react-router-devtools";

const queryClient = new QueryClient();

export const Route = createRootRoute({
  component: RootComponent,
});

function RootComponent() {
  return (
    <QueryClientProvider client={queryClient}>
      <div className="bg-background min-h-screen">
        <nav className="bg-card border-b">
          <div className="mx-auto flex max-w-7xl items-center gap-6 px-4 py-3">
            <Link
              to="/"
              className="text-foreground hover:text-primary text-lg font-semibold"
            >
              Home
            </Link>
            <Link
              to="/users"
              className="text-muted-foreground hover:text-foreground [&.active]:text-primary"
            >
              Users (Query + Table)
            </Link>
          </div>
        </nav>
        <main className="mx-auto max-w-7xl p-4">
          <Outlet />
        </main>
      </div>
      <ReactQueryDevtools />
      <TanStackRouterDevtools />
    </QueryClientProvider>
  );
}
