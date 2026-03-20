import { beforeEach, describe, expect, it, vi } from "vitest";

type DatasetDetailCommand = (id: number) => Promise<unknown>;
type ListDatasetsCommand = (options: unknown) => Promise<unknown>;

const { datasetDetailCommandMock, listDatasetsCommandMock } = vi.hoisted(
  () => ({
    datasetDetailCommandMock: vi.fn<DatasetDetailCommand>(),
    listDatasetsCommandMock: vi.fn<ListDatasetsCommand>(),
  }),
);

vi.mock("@/shared/lib/bindings", () => ({
  commands: {
    datasetDetail: (id: number) => datasetDetailCommandMock(id),
    listDatasets: (options: unknown) => listDatasetsCommandMock(options),
  },
}));

import { getDatasetDetail, listDatasets } from "./client";

describe("dataset client", () => {
  beforeEach(() => {
    datasetDetailCommandMock.mockReset();
    listDatasetsCommandMock.mockReset();
  });

  it("trims query options and normalizes dataset rows", async () => {
    listDatasetsCommandMock.mockResolvedValue({
      status: "ok",
      data: [
        {
          id: 1,
          name: "Dataset Alpha",
          description: "Alpha",
          favorite: false,
          tags: ["vision"],
          status: "Completed",
          createdAt: "2026-01-01T00:00:00Z",
          trashedAt: null,
        },
      ],
    });

    const result = await listDatasets({
      search: "  alpha  ",
      tags: [],
      favoriteOnly: false,
      statuses: [],
      sortBy: "id",
      sortDir: "desc",
      limit: 10,
      offset: 0,
    });

    expect(listDatasetsCommandMock).toHaveBeenCalledWith({
      search: "alpha",
      tags: undefined,
      favoriteOnly: undefined,
      statuses: undefined,
      sortBy: "id",
      sortDir: "desc",
      limit: 10,
      offset: 0,
    });
    expect(result).toEqual([
      expect.objectContaining({
        id: 1,
        name: "Dataset Alpha",
        createdAt: new Date("2026-01-01T00:00:00Z"),
      }),
    ]);
  });

  it("normalizes dataset detail responses", async () => {
    datasetDetailCommandMock.mockResolvedValue({
      status: "ok",
      data: {
        id: 7,
        name: "Dataset 7",
        description: "Details",
        favorite: false,
        tags: [],
        status: "Completed",
        createdAt: "2026-01-02T03:04:05Z",
        trashedAt: null,
        columns: [],
      },
    });

    const result = await getDatasetDetail(7);

    expect(datasetDetailCommandMock).toHaveBeenCalledWith(7);
    expect(result.createdAt).toBeInstanceOf(Date);
    expect(result.createdAt.toISOString()).toBe("2026-01-02T03:04:05.000Z");
  });

  it("propagates dataset command error envelopes", async () => {
    listDatasetsCommandMock.mockResolvedValue({
      status: "error",
      error: { message: "dataset listing failed" },
    });

    await expect(listDatasets()).rejects.toThrow("dataset listing failed");
  });
});
