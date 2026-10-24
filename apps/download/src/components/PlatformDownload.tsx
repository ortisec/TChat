import { Badge, Button, Card } from "@tchat/ui";
import { useEffect, useState } from "react";

type Os = "windows" | "macos" | "linux";

interface Platform {
  id: Os;
  name: string;
  detail: string;
  asset: string;
}

const platforms: Platform[] = [
  {
    id: "windows",
    name: "Windows",
    detail: "x86_64 · MSVC · Windows 10 o superior",
    asset: "tchat-x86_64-pc-windows-msvc.zip",
  },
  {
    id: "macos",
    name: "macOS",
    detail: "Apple Silicon y Intel · macOS 12 o superior",
    asset: "tchat-aarch64-apple-darwin.tar.gz",
  },
  {
    id: "linux",
    name: "Linux",
    detail: "x86_64 · binario estático (musl)",
    asset: "tchat-x86_64-unknown-linux-musl.tar.gz",
  },
];

function detectOs(userAgent: string): Os | null {
  const ua = userAgent.toLowerCase();
  if (ua.includes("win")) return "windows";
  if (ua.includes("mac")) return "macos";
  if (ua.includes("linux") || ua.includes("x11")) return "linux";
  return null;
}

export default function PlatformDownload() {
  const [detected, setDetected] = useState<Os | null>(null);
  const releasesUrl =
    import.meta.env.PUBLIC_RELEASES_URL ?? "https://github.com/ortisec/TChat/releases/latest";

  useEffect(() => {
    setDetected(detectOs(navigator.userAgent));
  }, []);

  return (
    <div className="grid gap-4 sm:grid-cols-3">
      {platforms.map((platform) => {
        const isDetected = detected === platform.id;
        return (
          <Card
            key={platform.id}
            className={isDetected ? "border-neutral-500 bg-neutral-900/70" : undefined}
          >
            <div className="flex items-start justify-between gap-3">
              <h2 className="text-base font-semibold text-neutral-100">{platform.name}</h2>
              {isDetected && <Badge>Tu sistema</Badge>}
            </div>
            <p className="mt-2 text-sm text-neutral-400">{platform.detail}</p>
            <p className="mt-4 break-all font-mono text-xs text-neutral-600">{platform.asset}</p>
            <Button
              href={releasesUrl}
              variant={isDetected ? "primary" : "secondary"}
              size="sm"
              className="mt-5 w-full"
            >
              Descargar
            </Button>
          </Card>
        );
      })}
    </div>
  );
}
