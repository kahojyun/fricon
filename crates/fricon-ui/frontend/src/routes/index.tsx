import { createFileRoute } from "@tanstack/react-router";
import { DatasetExplorerScreen } from "@/app/ui/DatasetExplorerScreen";

export const Route = createFileRoute("/")({
  component: HomeComponent,
});

function HomeComponent() {
  return <DatasetExplorerScreen />;
}
