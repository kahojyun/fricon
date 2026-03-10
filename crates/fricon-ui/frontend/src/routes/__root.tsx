import { useEffect } from "react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { ReactQueryDevtools } from "@tanstack/react-query-devtools";
import { createRootRoute, Link, Outlet } from "@tanstack/react-router";
import { TanStackRouterDevtools } from "@tanstack/react-router-devtools";
import { Database, Info } from "lucide-react";
import { Button } from "@/components/ui/button";
import { useWorkspaceInfoQuery } from "@/features/workspace";

const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      refetchOnWindowFocus: false,
      refetchOnReconnect: false,
    },
  },
});

export const Route = createRootRoute({
  component: RootComponent,
});

function RootComponent() {
  useEffect(() => {
    const media = window.matchMedia("(prefers-color-scheme: dark)");
    const apply = (isDark: boolean) => {
      document.documentElement.classList.toggle("dark", isDark);
    };
    apply(media.matches);
    const handler = (event: MediaQueryListEvent) => apply(event.matches);
    media.addEventListener("change", handler);
    return () => media.removeEventListener("change", handler);
  }, []);

  return (
    <QueryClientProvider client={queryClient}>
      <RootLayout />
      <ReactQueryDevtools />
      <TanStackRouterDevtools />
    </QueryClientProvider>
  );
}

function RootLayout() {
  const workspaceInfoQuery = useWorkspaceInfoQuery();
  const workspacePath = workspaceInfoQuery.data?.path ?? "(no workspace)";

  return (
    <div className="flex h-full flex-col bg-background text-foreground">
      <div className="flex flex-1 overflow-hidden">
        <aside className="flex w-14 flex-col items-center gap-2 border-r bg-muted/40 py-2">
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

        <div className="flex flex-1 flex-col overflow-hidden">
          <header className="flex h-9 shrink-0 items-center justify-between border-b bg-muted/30 px-3">
            <span className="text-xs font-semibold tracking-tight">fricon</span>
            <span className="truncate text-[11px] text-muted-foreground">
              {workspacePath}
            </span>
          </header>

          <main className="flex-1 overflow-hidden">
            <Outlet />
          </main>
        </div>
      </div>

      <footer className="flex h-6 items-center border-t bg-muted/40 px-3 text-[11px] text-muted-foreground">
        <span className="truncate">Ready</span>
      </footer>
    </div>
  );
}
