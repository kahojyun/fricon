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
