import { createFileRoute } from "@tanstack/react-router";
import { DataViewer } from "@/components/data-viewer";

export const Route = createFileRoute("/")({
  component: HomeComponent,
});

function HomeComponent() {
  return <DataViewer />;
}
