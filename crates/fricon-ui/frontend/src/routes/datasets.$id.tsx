import { createFileRoute } from "@tanstack/react-router";
import { DataViewer } from "@/components/data-viewer";

export const Route = createFileRoute("/datasets/$id")({
  component: DatasetRouteComponent,
});

function DatasetRouteComponent() {
  const { id } = Route.useParams();
  return <DataViewer datasetId={id} />;
}
