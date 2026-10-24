import { onAuthStateChanged, type User } from "firebase/auth";
import { useEffect, useState } from "react";

import { firebaseConfigured, getFirebaseAuth } from "../lib/firebase";

interface AuthUserState {
  user: User | null;
  ready: boolean;
}

/** Observa la sesión de Firebase. No ejecuta nada durante el render del servidor. */
export function useAuthUser(): AuthUserState {
  const [user, setUser] = useState<User | null>(null);
  const [ready, setReady] = useState(false);

  useEffect(() => {
    if (!firebaseConfigured) {
      setReady(true);
      return;
    }

    return onAuthStateChanged(getFirebaseAuth(), (nextUser) => {
      setUser(nextUser);
      setReady(true);
    });
  }, []);

  return { user, ready };
}
