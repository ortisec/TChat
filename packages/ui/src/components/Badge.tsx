import type { ReactNode } from "react";

import { cn } from "../cn";

export function Badge({ className, children }: { className?: string; children: ReactNode }) {
  return (
    <span
      className={cn(
        "inline-flex items-center gap-1.5 rounded-full border border-neutral-700 px-2.5 py-0.5",
        "font-mono text-xs text-neutral-300",
        className,
      )}
    >
      {children}
    </span>
  );
}
