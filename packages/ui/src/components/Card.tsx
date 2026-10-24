import type { ReactNode } from "react";

import { cn } from "../cn";

export function Card({ className, children }: { className?: string; children: ReactNode }) {
  return (
    <div className={cn("rounded-card border border-neutral-800 bg-neutral-900/40 p-6", className)}>
      {children}
    </div>
  );
}

export function CardTitle({ className, children }: { className?: string; children: ReactNode }) {
  return <h3 className={cn("text-base font-semibold text-neutral-100", className)}>{children}</h3>;
}

export function CardDescription({
  className,
  children,
}: {
  className?: string;
  children: ReactNode;
}) {
  return (
    <p className={cn("mt-2 text-sm leading-relaxed text-neutral-400", className)}>{children}</p>
  );
}
