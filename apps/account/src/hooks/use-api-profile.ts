import type { components } from "@tchat/api-client";
import type { User } from "firebase/auth";
import { useEffect, useState } from "react";

import { api } from "../lib/api";

export type UserPublic = components["schemas"]["UserPublic"];
export type SyncState = "idle" | "syncing" | "ready" | "error";

/** Sincroniza el usuario de Firebase con la API y devuelve su perfil público. */
export function useApiProfile(user: User | null): {
  profile: UserPublic | null;
  state: SyncState;
} {
  const [profile, setProfile] = useState<UserPublic | null>(null);
  const [state, setState] = useState<SyncState>("idle");

  useEffect(() => {
    if (!user) {
      setProfile(null);
      setState("idle");
      return;
    }

    let cancelled = false;
    setState("syncing");

    void (async () => {
      try {
        const token = await user.getIdToken();
        const { data, error } = await api.POST("/api/v1/auth/sync", {
          headers: { Authorization: `Bearer ${token}` },
        });

        if (cancelled) {
          return;
        }

        if (error) {
          setState("error");
          return;
        }

        setProfile(data ?? null);
        setState("ready");
      } catch {
        if (!cancelled) {
          setState("error");
        }
      }
    })();

    return () => {
      cancelled = true;
    };
  }, [user]);

  return { profile, state };
}
