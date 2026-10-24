import type { ComponentPropsWithoutRef, ReactNode } from "react";

import { cn } from "../cn";

export function Input({ className, ...props }: ComponentPropsWithoutRef<"input">) {
  return (
    <input
      className={cn(
        "h-11 w-full rounded-md border border-neutral-700 bg-neutral-950 px-3 text-sm",
        "text-neutral-100 placeholder:text-neutral-500",
        "focus:border-neutral-400 focus:outline-none",
        className,
      )}
      {...props}
    />
  );
}

export function Label({
  className,
  children,
  ...props
}: ComponentPropsWithoutRef<"label"> & { children: ReactNode }) {
  return (
    <label
      className={cn("mb-1.5 block text-sm font-medium text-neutral-300", className)}
      {...props}
    >
      {children}
    </label>
  );
}

export function FieldError({ children }: { children: ReactNode }) {
  return (
    <p role="alert" className="mt-2 text-sm text-neutral-300">
      <span aria-hidden="true" className="mr-1.5 font-mono">
        [!]
      </span>
      {children}
    </p>
  );
}
