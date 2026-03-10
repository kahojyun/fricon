import { createFileRoute } from "@tanstack/react-router";
import { DataViewer } from "@/features/data-explorer";

export const Route = createFileRoute("/")({
  component: HomeComponent,
});

function HomeComponent() {
  return <DataViewer />;
}
