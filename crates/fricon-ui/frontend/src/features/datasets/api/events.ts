import { getCurrentWindow } from "@tauri-apps/api/window";
import { type DatasetChangeKind, events } from "@/shared/lib/bindings";
import { normalizeDataset, type DatasetInfo } from "./types";

export interface DatasetChangedEvent {
  info: DatasetInfo | null;
  kind: DatasetChangeKind;
}

export function onDatasetChanged(
  callback: (event: DatasetChangedEvent) => void,
) {
  return events.datasetChanged.listen((event) => {
    callback({
      info:
        event.payload.info !== null
          ? normalizeDataset(event.payload.info)
          : null,
      kind: event.payload.kind,
    });
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
