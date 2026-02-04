import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { ReactQueryDevtools } from "@tanstack/react-query-devtools";
import { createRootRoute, Link, Outlet } from "@tanstack/react-router";
import { TanStackRouterDevtools } from "@tanstack/react-router-devtools";
import { Database, Info } from "lucide-react";
import { Button } from "@/components/ui/button";
import { useWorkspaceStore } from "@/lib/useWorkspaceStore";

const queryClient = new QueryClient();

export const Route = createRootRoute({
  component: RootComponent,
});

function RootComponent() {
  const workspacePath = useWorkspaceStore((state) => state.path);

  return (
    <QueryClientProvider client={queryClient}>
      <div className="bg-background text-foreground flex min-h-screen flex-col">
        <div className="flex flex-1 overflow-hidden">
          <aside className="bg-muted/40 flex w-14 flex-col items-center gap-2 border-r py-2">
            <Button
              variant="outline"
              size="icon"
              className="data-[active=true]:bg-primary/10 data-[active=true]:text-primary"
              nativeButton={false}
              render={(props) => (
                <Link
                  {...props}
                  to="/"
                  aria-label="Data"
                  activeProps={{ "data-active": "true" }}
                >
                  <Database />
                </Link>
              )}
            />
            <Button
              variant="outline"
              size="icon"
              className="data-[active=true]:bg-primary/10 data-[active=true]:text-primary"
              nativeButton={false}
              render={(props) => (
                <Link
                  {...props}
                  to="/credits"
                  aria-label="Credits"
                  activeProps={{ "data-active": "true" }}
                >
                  <Info />
                </Link>
              )}
            />
          </aside>

          <main className="flex-1 overflow-y-auto">
            <Outlet />
          </main>
        </div>

        <footer className="bg-muted/60 flex h-8 items-center px-3 text-xs">
          <div className="truncate">Workspace: {workspacePath}</div>
        </footer>
      </div>
      <ReactQueryDevtools />
      <TanStackRouterDevtools />
    </QueryClientProvider>
  );
}
