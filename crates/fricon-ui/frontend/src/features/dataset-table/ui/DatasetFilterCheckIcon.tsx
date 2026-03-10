import { Check } from "lucide-react";
import { cn } from "@/shared/lib/utils";

export function DatasetFilterCheckIcon({ active }: { active: boolean }) {
  return (
    <Check
      className={cn(
        "size-3",
        active ? "text-foreground opacity-100" : "text-transparent opacity-0",
      )}
    />
  );
}
