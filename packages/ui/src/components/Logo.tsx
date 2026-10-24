import { cn } from "../cn";

export function Logo({ className }: { className?: string }) {
  return (
    <span className={cn("inline-flex items-center gap-2.5", className)}>
      <svg
        aria-hidden="true"
        viewBox="0 0 24 24"
        className="h-7 w-7"
        fill="none"
        stroke="currentColor"
        strokeWidth="1.75"
        strokeLinecap="round"
        strokeLinejoin="round"
      >
        <rect x="2.5" y="4" width="19" height="15" rx="3" />
        <path d="m8 10 2 2-2 2" />
        <path d="M13 14h3" />
      </svg>
      <span className="text-lg font-semibold tracking-tight text-neutral-100">TChat</span>
    </span>
  );
}
