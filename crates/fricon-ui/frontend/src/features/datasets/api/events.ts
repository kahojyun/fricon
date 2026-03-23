import { getCurrentWindow } from "@tauri-apps/api/window";
import { events } from "@/shared/lib/bindings";
import { normalizeDataset, type DatasetInfo } from "./types";

export function onDatasetCreated(callback: (event: DatasetInfo) => void) {
  return events.datasetCreated.listen((event) => {
    callback(normalizeDataset(event.payload));
  });
}

export function onDatasetUpdated(callback: (event: DatasetInfo) => void) {
  return events.datasetUpdated.listen((event) => {
    callback(normalizeDataset(event.payload));
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
