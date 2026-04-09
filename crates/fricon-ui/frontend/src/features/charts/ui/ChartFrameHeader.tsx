import type { ChartFrameHeaderData } from "./chartFrameHeaderModel";

interface ChartFrameHeaderProps {
  header: ChartFrameHeaderData;
}

export function ChartFrameHeader({ header }: ChartFrameHeaderProps) {
  return (
    <div className="max-w-[min(48rem,80%)] min-w-0 rounded-md bg-background/72 px-3 py-1.5 text-center shadow-sm backdrop-blur-[1px]">
      <div className="text-sm leading-tight font-semibold text-foreground">
        {header.title}
      </div>
      {header.meta.length > 0 ? (
        <div className="mt-0.5 flex flex-wrap items-center justify-center gap-x-2 gap-y-0.5 text-[11px] text-muted-foreground">
          {header.meta.map((item) => (
            <span key={item}>{item}</span>
          ))}
        </div>
      ) : null}
    </div>
  );
}
