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
          deletedAt: null,
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
        trashedAt: null,
        deletedAt: null,
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
        trashedAt: "2026-01-03T04:05:06Z",
        deletedAt: null,
        payloadAvailable: true,
        columns: [
          {
            name: "signal",
            label: "Signal",
            unit: "V",
            isComplex: false,
            isTrace: false,
            isInferredAxis: false,
            hiddenByDefault: true,
            isChartAxisCandidate: true,
          },
        ],
        chartSemantics: {
          duplicatePolicy: "latest_by_record_id",
          indexRealization: "implicit",
          axes: [
            {
              id: "logicalIndex:gate",
              name: "gate",
              label: "Gate",
              kind: "logical_index",
              numeric: true,
              isInferredAxis: false,
              physicalColumn: null,
            },
          ],
          valueColumns: [
            {
              id: "column:signal",
              name: "signal",
              label: "Signal",
              isComplex: false,
              isTrace: false,
              hiddenByDefault: true,
            },
          ],
          chartAxisCandidates: [],
        },
      },
    });

    const result = await getDatasetDetail(7);

    expect(datasetDetailCommandMock).toHaveBeenCalledWith(7);
    expect(result.createdAt).toBeInstanceOf(Date);
    expect(result.createdAt.toISOString()).toBe("2026-01-02T03:04:05.000Z");
    expect(result.trashedAt).toBeInstanceOf(Date);
    expect(result.trashedAt?.toISOString()).toBe("2026-01-03T04:05:06.000Z");
    expect(result.deletedAt).toBeNull();
    expect(result.payloadAvailable).toBe(true);
    expect(result.columns).toEqual([
      {
        name: "signal",
        label: "Signal",
        unit: "V",
        isComplex: false,
        isTrace: false,
        isInferredAxis: false,
        hiddenByDefault: true,
        isChartAxisCandidate: true,
      },
    ]);
    expect(result.chartSemantics).toEqual({
      duplicatePolicy: "latest_by_record_id",
      indexRealization: "implicit",
      axes: [
        {
          id: "logicalIndex:gate",
          name: "gate",
          label: "Gate",
          kind: "logical_index",
          numeric: true,
          isInferredAxis: false,
          physicalColumn: null,
        },
      ],
      valueColumns: [
        {
          id: "column:signal",
          name: "signal",
          label: "Signal",
          isComplex: false,
          isTrace: false,
          hiddenByDefault: true,
        },
      ],
      chartAxisCandidates: [],
    });
  });

  it("normalizes absent chart semantics to null", async () => {
    datasetDetailCommandMock.mockResolvedValue({
      status: "ok",
      data: {
        id: 8,
        name: "Deleted Payload",
        description: "",
        favorite: false,
        tags: [],
        status: "Completed",
        createdAt: "2026-01-02T03:04:05Z",
        trashedAt: null,
        deletedAt: null,
        payloadAvailable: false,
        columns: [],
      },
    });

    const result = await getDatasetDetail(8);

    expect(result.chartSemantics).toBeNull();
  });

  it("propagates dataset command error envelopes", async () => {
    listDatasetsCommandMock.mockResolvedValue({
      status: "error",
      error: { code: "dataset_not_found", message: "dataset listing failed" },
    });

    await expect(listDatasets()).rejects.toThrow(
      "[dataset_not_found] dataset listing failed",
    );
  });
});
