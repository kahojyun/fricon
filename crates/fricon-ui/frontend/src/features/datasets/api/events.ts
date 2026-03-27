import { getCurrentWindow } from "@tauri-apps/api/window";
import {
  events,
  type DatasetChanged as WireDatasetChanged,
} from "@/shared/lib/bindings";
import { normalizeDataset, type DatasetInfo } from "./types";

type WireDatasetChangedWithInfo = Extract<
  WireDatasetChanged,
  { info: unknown }
>;

/** Normalized dataset change event (info.createdAt etc. are Date objects). */
export type DatasetChangedEvent =
  | { kind: WireDatasetChangedWithInfo["kind"]; info: DatasetInfo }
  | { kind: "globalTagsChanged" };

export function onDatasetChanged(
  callback: (event: DatasetChangedEvent) => void,
) {
  return events.datasetChanged.listen((event) => {
    const p = event.payload;
    if (p.kind === "globalTagsChanged") {
      callback(p);
    } else {
      callback({ kind: p.kind, info: normalizeDataset(p.info) });
    }
  });
}

export function onDatasetArchiveDrop(
  callback: (archivePaths: string[]) => void,
) {
  return getCurrentWindow().onDragDropEvent((event) => {
    if (event.payload.type !== "drop") {
      return;
    }

    const archivePaths = event.payload.paths.filter((path) =>
      path.endsWith(".tar.zst"),
    );
    if (archivePaths.length > 0) {
      callback(archivePaths);
    }
  });
}
