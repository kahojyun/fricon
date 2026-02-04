import * as React from "react";

import { cn } from "@/lib/utils";

const Input = ({
  ref,
  className,
  type = "text",
  ...props
}: React.ComponentProps<"input"> & {
  ref?: React.RefObject<HTMLInputElement | null>;
}) => (
  <input
    ref={ref}
    type={type}
    className={cn(
      "border-input bg-background text-foreground placeholder:text-muted-foreground focus-visible:ring-ring flex h-9 w-full rounded-md border px-3 py-1 text-sm shadow-sm focus-visible:ring-2 focus-visible:outline-none disabled:cursor-not-allowed disabled:opacity-50",
      className,
    )}
    {...props}
  />
);
Input.displayName = "Input";

export { Input };
