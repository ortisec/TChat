import { Badge, Button, Card } from "@tchat/ui";
import { useEffect, useState } from "react";

type Os = "windows" | "macos" | "linux";

interface ReleaseAsset {
  name: string;
  url: string;
  sha256?: string | null;
}

interface LatestRelease {
  version: string;
  tag?: string;
  released_at?: string;
  assets: ReleaseAsset[];
}

interface Platform {
  id: Os;
  name: string;
  detail: string;
  targets: string[];
  expected: string;
}

const platforms: Platform[] = [
  {
    id: "windows",
    name: "Windows",
    detail: "x86_64 · MSVC · Windows 10 o superior",
    targets: ["x86_64-pc-windows-msvc"],
    expected: "tchat-cli-x86_64-pc-windows-msvc.zip",
  },
  {
    id: "macos",
    name: "macOS",
    detail: "Apple Silicon e Intel · macOS 12 o superior",
    targets: ["aarch64-apple-darwin", "x86_64-apple-darwin"],
    expected: "tchat-cli-aarch64-apple-darwin.tar.xz",
  },
  {
    id: "linux",
    name: "Linux",
    detail: "x86_64 · glibc y musl",
    targets: ["x86_64-unknown-linux-gnu", "x86_64-unknown-linux-musl"],
    expected: "tchat-cli-x86_64-unknown-linux-musl.tar.xz",
  },
];

function detectOs(userAgent: string): Os | null {
  const ua = userAgent.toLowerCase();
  if (ua.includes("win")) return "windows";
  if (ua.includes("mac")) return "macos";
  if (ua.includes("linux") || ua.includes("x11")) return "linux";
  return null;
}

function assetLabel(name: string): string {
  if (name.endsWith(".msi")) return "Instalador MSI";
  if (name.endsWith(".zip")) return "ZIP portable";
  if (name.includes("aarch64-apple")) return "Apple Silicon";
  if (name.includes("x86_64-apple")) return "Intel";
  if (name.includes("linux-musl")) return "musl";
  if (name.includes("linux-gnu")) return "glibc";
  if (name.includes("windows")) return "x86_64";
  return name;
}

export default function PlatformDownload() {
  const [detected, setDetected] = useState<Os | null>(null);
  const [latest, setLatest] = useState<LatestRelease | null>(null);
  const [status, setStatus] = useState<"loading" | "ready" | "error">("loading");

  const releasesUrl =
    import.meta.env.PUBLIC_RELEASES_URL ?? "https://github.com/ortisec/TChat/releases/latest";

  useEffect(() => {
    setDetected(detectOs(navigator.userAgent));

    let cancelled = false;

    fetch("/latest.json", { headers: { Accept: "application/json" } })
      .then((response) => (response.ok ? response.json() : Promise.reject(response.status)))
      .then((data: LatestRelease) => {
        if (!cancelled) {
          setLatest(data);
          setStatus("ready");
        }
      })
      .catch(() => {
        if (!cancelled) {
          setStatus("error");
        }
      });

    return () => {
      cancelled = true;
    };
  }, []);

  return (
    <div>
      <p className="mb-4 font-mono text-xs text-neutral-500">
        {status === "ready" && latest
          ? `Última versión: v${latest.version}`
          : status === "error"
            ? "Descargas disponibles en GitHub Releases"
            : "Buscando la última versión…"}
      </p>

      <div className="grid gap-4 sm:grid-cols-3">
        {platforms.map((platform) => {
          const isDetected = detected === platform.id;
          const assets =
            latest?.assets.filter((asset) =>
              platform.targets.some((target) => asset.name.includes(target)),
            ) ?? [];

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
              <p className="mt-4 break-all font-mono text-xs text-neutral-600">
                {assets[0]?.name ?? platform.expected}
              </p>

              <div className="mt-5 flex flex-col gap-2">
                {assets.length > 0 ? (
                  assets.map((asset) => (
                    <Button
                      key={asset.name}
                      href={asset.url}
                      variant={isDetected ? "primary" : "secondary"}
                      size="sm"
                      className="w-full"
                    >
                      Descargar · {assetLabel(asset.name)}
                    </Button>
                  ))
                ) : (
                  <Button
                    href={releasesUrl}
                    variant={isDetected ? "primary" : "secondary"}
                    size="sm"
                    className="w-full"
                  >
                    Ver releases
                  </Button>
                )}
              </div>
            </Card>
          );
        })}
      </div>
    </div>
  );
}
