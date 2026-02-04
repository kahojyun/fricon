import * as React from "react";
import { Switch } from "@base-ui/react/switch";

import { cn } from "@/lib/utils";

const SwitchRoot = ({
  ref,
  className,
  ...props
}: React.ComponentPropsWithoutRef<typeof Switch.Root> & {
  ref?: React.RefObject<React.ElementRef<typeof Switch.Root> | null>;
}) => (
  <Switch.Root
    ref={ref}
    className={cn(
      "peer data-[checked]:bg-primary data-[unchecked]:bg-muted border-border focus-visible:ring-ring inline-flex h-5 w-9 shrink-0 cursor-pointer items-center rounded-full border transition-colors focus-visible:ring-2 focus-visible:outline-none",
      className,
    )}
    {...props}
  >
    <Switch.Thumb className="bg-background pointer-events-none block size-4 translate-x-0.5 rounded-full shadow transition-transform data-[checked]:translate-x-4" />
  </Switch.Root>
);
SwitchRoot.displayName = "Switch";

export { SwitchRoot as Switch };
