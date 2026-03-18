import type { ReactNode } from "react";
import {
  RouterProvider,
  createMemoryHistory,
  createRouter,
} from "@tanstack/react-router";
import { act, render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { clearMocks, mockIPC } from "@tauri-apps/api/mocks";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { routeTree } from "@/routeTree.gen";

const { datasetCreatedListenMock, datasetUpdatedListenMock } = vi.hoisted(
  () => ({
    datasetCreatedListenMock: vi.fn(),
    datasetUpdatedListenMock: vi.fn(),
  }),
);

vi.mock("@/shared/lib/bindings", async (importOriginal) => {
  const actual = await importOriginal<typeof import("@/shared/lib/bindings")>();
  return {
    ...actual,
    events: {
      datasetCreated: {
        listen: datasetCreatedListenMock,
      },
      datasetUpdated: {
        listen: datasetUpdatedListenMock,
      },
    },
  };
});

vi.mock("@tanstack/react-query-devtools", () => ({
  ReactQueryDevtools: () => null,
}));

vi.mock("@tanstack/react-router-devtools", () => ({
  TanStackRouterDevtools: () => null,
}));

vi.mock("@tanstack/react-virtual", () => ({
  useVirtualizer: ({ count }: { count: number }) => ({
    getTotalSize: () => count * 36,
    getVirtualItems: () =>
      Array.from({ length: count }, (_, index) => ({
        index,
        start: index * 36,
        end: (index + 1) * 36,
      })),
    measureElement: () => undefined,
    scrollToIndex: () => undefined,
  }),
}));

vi.mock("react-resizable-panels", () => ({
  Group: ({
    children,
    ...props
  }: {
    children: ReactNode;
    ["aria-orientation"]?: "horizontal" | "vertical";
  }) => <div {...props}>{children}</div>,
  Panel: ({ children }: { children: ReactNode }) => <div>{children}</div>,
  Separator: (props: { ["aria-orientation"]?: "horizontal" | "vertical" }) => (
    <div {...props} />
  ),
}));

describe("router app shell", () => {
  beforeEach(() => {
    clearMocks();
    datasetCreatedListenMock.mockReset();
    datasetUpdatedListenMock.mockReset();
    datasetCreatedListenMock.mockResolvedValue(() => undefined);
    datasetUpdatedListenMock.mockResolvedValue(() => undefined);
    if (typeof window.localStorage?.clear === "function") {
      window.localStorage.clear();
    }
    Element.prototype.scrollIntoView = vi.fn();

    mockIPC((cmd) => {
      switch (cmd) {
        case "get_workspace_info":
          return { path: "/tmp/fricon-workspace" };
        case "list_datasets":
          return [
            {
              id: 1,
              name: "Dataset Alpha",
              description: "Alpha",
              favorite: false,
              tags: ["vision"],
              status: "Completed",
              createdAt: "2026-01-01T00:00:00Z",
            },
          ];
        case "list_dataset_tags":
          return ["vision"];
        default:
          return null;
      }
    });
  });

  afterEach(() => {
    clearMocks();
  });

  it("renders the real app shell and navigates between root and credits", async () => {
    const user = userEvent.setup();
    const router = createRouter({
      routeTree,
      history: createMemoryHistory({
        initialEntries: ["/"],
      }),
    });

    await act(async () => {
      render(<RouterProvider router={router} />);
      await router.load();
    });

    expect(await screen.findByText("No dataset selected")).toBeInTheDocument();
    expect(
      screen.getByPlaceholderText("Filter datasets..."),
    ).toBeInTheDocument();
    expect(screen.getByText("/tmp/fricon-workspace")).toBeInTheDocument();

    const dataLink = screen.getByRole("button", { name: "Data" });
    const creditsLink = screen.getByRole("button", { name: "Credits" });
    expect(dataLink).toHaveAttribute("data-active", "true");
    expect(creditsLink).not.toHaveAttribute("data-active", "true");
    expect(dataLink).toHaveAttribute("href", "/");
    expect(creditsLink).toHaveAttribute("href", "/credits");
    expect(router.state.location.pathname).toBe("/");

    await user.click(creditsLink);

    expect(
      await screen.findByRole("button", {
        name: "Computer icons created by Freepik - Flaticon",
      }),
    ).toBeInTheDocument();
    await waitFor(() => {
      expect(screen.getByRole("button", { name: "Credits" })).toHaveAttribute(
        "data-active",
        "true",
      );
    });
    expect(router.state.location.pathname).toBe("/credits");

    await user.click(screen.getByRole("button", { name: "Data" }));

    expect(await screen.findByText("No dataset selected")).toBeInTheDocument();
    await waitFor(() => {
      expect(screen.getByRole("button", { name: "Data" })).toHaveAttribute(
        "data-active",
        "true",
      );
    });
    expect(router.state.location.pathname).toBe("/");
  });
});
