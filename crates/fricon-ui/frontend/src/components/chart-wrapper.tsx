import type { ChartOptions } from "@/lib/chartTypes";

interface ChartWrapperProps {
  data?: ChartOptions;
}

export function ChartWrapper({ data }: ChartWrapperProps) {
  if (!data) {
    return (
      <div className="text-muted-foreground flex h-full items-center justify-center text-sm">
        No chart data
      </div>
    );
  }

  return (
    <div className="bg-muted/30 flex h-full items-center justify-center rounded-md border text-sm">
      Chart placeholder: {data.type}
    </div>
  );
}
