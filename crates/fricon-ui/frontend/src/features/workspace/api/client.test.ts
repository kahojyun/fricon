import { beforeEach, describe, expect, it, vi } from "vitest";

type GetWorkspaceInfoCommand = () => Promise<unknown>;

const { getWorkspaceInfoCommandMock } = vi.hoisted(() => ({
  getWorkspaceInfoCommandMock: vi.fn<GetWorkspaceInfoCommand>(),
}));

vi.mock("@/shared/lib/bindings", () => ({
  commands: {
    getWorkspaceInfo: () => getWorkspaceInfoCommandMock(),
  },
}));

import { getWorkspaceInfo } from "./client";

describe("workspace client", () => {
  beforeEach(() => {
    getWorkspaceInfoCommandMock.mockReset();
  });

  it("returns workspace info from the Tauri command", async () => {
    getWorkspaceInfoCommandMock.mockResolvedValue({
      status: "ok",
      data: { path: "/tmp/workspace" },
    });

    await expect(getWorkspaceInfo()).resolves.toEqual({
      path: "/tmp/workspace",
    });
  });

  it("propagates backend error envelopes", async () => {
    getWorkspaceInfoCommandMock.mockResolvedValue({
      status: "error",
      error: { code: "workspace", message: "workspace unavailable" },
    });

    await expect(getWorkspaceInfo()).rejects.toThrow(
      "[workspace] workspace unavailable",
    );
  });
});
