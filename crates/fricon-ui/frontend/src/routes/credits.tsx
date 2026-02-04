import { createFileRoute } from "@tanstack/react-router";
import { Button } from "@/components/ui/button";

export const Route = createFileRoute("/credits")({
  component: CreditsComponent,
});

function CreditsComponent() {
  return (
    <div className="p-4">
      <Button
        nativeButton={false}
        variant="link"
        render={(props) => (
          <a
            {...props}
            href="https://www.flaticon.com/free-icons/computer"
            target="_blank"
            rel="noreferrer"
          >
            Computer icons created by Freepik - Flaticon
          </a>
        )}
      />
    </div>
  );
}
