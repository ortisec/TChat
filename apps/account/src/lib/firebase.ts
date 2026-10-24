import { getApps, initializeApp, type FirebaseOptions } from "firebase/app";
import { getAuth, type Auth } from "firebase/auth";

const config: FirebaseOptions = {
  apiKey: import.meta.env.PUBLIC_FIREBASE_API_KEY,
  authDomain: import.meta.env.PUBLIC_FIREBASE_AUTH_DOMAIN,
  projectId: import.meta.env.PUBLIC_FIREBASE_PROJECT_ID,
  appId: import.meta.env.PUBLIC_FIREBASE_APP_ID,
  messagingSenderId: import.meta.env.PUBLIC_FIREBASE_SENDER_ID,
};

export const firebaseConfigured = Boolean(
  config.apiKey && config.authDomain && config.projectId && config.appId,
);

export const missingFirebaseEnv: string[] = (
  [
    ["PUBLIC_FIREBASE_API_KEY", config.apiKey],
    ["PUBLIC_FIREBASE_AUTH_DOMAIN", config.authDomain],
    ["PUBLIC_FIREBASE_PROJECT_ID", config.projectId],
    ["PUBLIC_FIREBASE_APP_ID", config.appId],
  ] as const
)
  .filter(([, value]) => !value)
  .map(([name]) => name);

let cachedAuth: Auth | null = null;

/** Devuelve la instancia de Auth. Solo debe llamarse en el navegador. */
export function getFirebaseAuth(): Auth {
  if (!firebaseConfigured) {
    throw new Error("Firebase no está configurado.");
  }

  if (!cachedAuth) {
    const app = getApps()[0] ?? initializeApp(config);
    cachedAuth = getAuth(app);
  }

  return cachedAuth;
}
