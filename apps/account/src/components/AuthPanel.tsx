import {
  Badge,
  Button,
  Card,
  CardDescription,
  CardTitle,
  FieldError,
  Input,
  Label,
} from "@tchat/ui";
import {
  createUserWithEmailAndPassword,
  GoogleAuthProvider,
  signInWithEmailAndPassword,
  signInWithPopup,
  signOut,
  updateProfile,
} from "firebase/auth";
import { useState } from "react";

import { useApiProfile, type SyncState, type UserPublic } from "../hooks/use-api-profile";
import { useAuthUser } from "../hooks/use-auth-user";
import { authErrorMessage } from "../lib/auth-errors";
import { firebaseConfigured, getFirebaseAuth, missingFirebaseEnv } from "../lib/firebase";

type Mode = "login" | "register";

function initials(name: string | null, email: string | null): string {
  const source = name?.trim() || email?.split("@")[0] || "?";
  return source.slice(0, 2).toUpperCase();
}

function providerLabel(providerId: string): string {
  if (providerId === "google.com") return "Google";
  if (providerId === "password") return "Correo y contraseña";
  return providerId;
}

function SetupNotice() {
  return (
    <Card>
      <Badge>Configuración pendiente</Badge>
      <CardTitle className="mt-4">Conecta tu proyecto de Firebase</CardTitle>
      <CardDescription>
        Faltan variables públicas en <span className="font-mono">apps/account/.env</span>. Cópialas
        desde Firebase Console → Project settings → General → Your apps → Web app.
      </CardDescription>
      <ul className="mt-4 space-y-1 font-mono text-xs text-neutral-400">
        {missingFirebaseEnv.map((name) => (
          <li key={name}>· {name}</li>
        ))}
      </ul>
    </Card>
  );
}

function ProfileView({
  ready,
  user,
  profile,
  syncState,
}: {
  ready: boolean;
  user: ReturnType<typeof useAuthUser>["user"];
  profile: UserPublic | null;
  syncState: SyncState;
}) {
  const [copied, setCopied] = useState(false);

  if (!ready) {
    return <p className="text-sm text-neutral-500">Comprobando sesión…</p>;
  }

  if (!user) {
    return <SetupNotice />;
  }

  const provider = user.providerData[0]?.providerId ?? "password";

  async function copyUid() {
    if (!user) return;
    await navigator.clipboard.writeText(user.uid);
    setCopied(true);
    window.setTimeout(() => setCopied(false), 1500);
  }

  return (
    <Card>
      <div className="flex items-center gap-4">
        <span className="flex h-12 w-12 items-center justify-center rounded-full border border-neutral-700 bg-neutral-900 font-mono text-sm text-neutral-200">
          {initials(user.displayName, user.email)}
        </span>
        <div className="min-w-0">
          <h1 className="truncate text-lg font-semibold">{user.displayName ?? "Sin nombre"}</h1>
          <p className="truncate text-sm text-neutral-400">{user.email}</p>
        </div>
      </div>

      <dl className="mt-6 divide-y divide-neutral-800 border-y border-neutral-800 text-sm">
        <div className="flex items-center justify-between gap-4 py-3">
          <dt className="text-neutral-500">Método</dt>
          <dd className="text-neutral-200">{providerLabel(provider)}</dd>
        </div>
        <div className="flex items-center justify-between gap-4 py-3">
          <dt className="text-neutral-500">Cuenta</dt>
          <dd className="text-neutral-300">
            {syncState === "ready" && profile ? (
              <span className="font-mono">@{profile.username}</span>
            ) : syncState === "error" ? (
              <span className="text-neutral-500">sin conexión con la API</span>
            ) : (
              <span className="text-neutral-500">sincronizando…</span>
            )}
          </dd>
        </div>
        <div className="flex items-center justify-between gap-4 py-3">
          <dt className="text-neutral-500">UID</dt>
          <dd className="flex min-w-0 items-center gap-2">
            <span className="truncate font-mono text-xs text-neutral-300">{user.uid}</span>
            <button
              type="button"
              onClick={copyUid}
              className="shrink-0 rounded border border-neutral-700 px-2 py-1 font-mono text-xs text-neutral-400 transition-colors hover:border-neutral-500 hover:text-neutral-100"
            >
              {copied ? "copiado" : "copiar"}
            </button>
          </dd>
        </div>
        <div className="flex items-center justify-between gap-4 py-3">
          <dt className="text-neutral-500">Alta</dt>
          <dd className="text-neutral-300">{user.metadata.creationTime ?? "—"}</dd>
        </div>
      </dl>

      <div className="mt-6 flex flex-col gap-2">
        <Button href="/cli-auth" variant="secondary">
          Vincular terminal
        </Button>
        <Button variant="ghost" onClick={() => void signOut(getFirebaseAuth())}>
          Cerrar sesión
        </Button>
      </div>

      <p className="mt-6 text-xs leading-relaxed text-neutral-600">
        La API de TChat guarda tu perfil, tus amistades y tus conversaciones. Si ves «sin conexión»,
        arranca el backend con <span className="font-mono">just api</span>.
      </p>
    </Card>
  );
}

export function AuthForm() {
  const [mode, setMode] = useState<Mode>("login");
  const [displayName, setDisplayName] = useState("");
  const [email, setEmail] = useState("");
  const [password, setPassword] = useState("");
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);

  async function handleSubmit(event: { preventDefault: () => void }) {
    event.preventDefault();
    setBusy(true);
    setError(null);

    try {
      const auth = getFirebaseAuth();
      if (mode === "register") {
        const credential = await createUserWithEmailAndPassword(auth, email, password);
        const name = displayName.trim();
        if (name) {
          await updateProfile(credential.user, { displayName: name });
        }
      } else {
        await signInWithEmailAndPassword(auth, email, password);
      }
    } catch (caught) {
      setError(authErrorMessage(caught));
    } finally {
      setBusy(false);
    }
  }

  async function handleGoogle() {
    setBusy(true);
    setError(null);

    try {
      await signInWithPopup(getFirebaseAuth(), new GoogleAuthProvider());
    } catch (caught) {
      setError(authErrorMessage(caught));
    } finally {
      setBusy(false);
    }
  }

  function switchMode(next: Mode) {
    setMode(next);
    setError(null);
  }

  return (
    <Card>
      <h1 className="text-xl font-semibold tracking-tight">
        {mode === "login" ? "Entra en TChat" : "Crea tu cuenta"}
      </h1>
      <p className="mt-2 text-sm text-neutral-400">
        {mode === "login"
          ? "Usa tu correo o tu cuenta de Google."
          : "Un correo y una contraseña. Nada más."}
      </p>

      <Button
        variant="secondary"
        className="mt-6 w-full"
        disabled={busy}
        onClick={() => void handleGoogle()}
      >
        <span
          aria-hidden="true"
          className="flex h-5 w-5 items-center justify-center rounded-full border border-neutral-600 font-mono text-[11px]"
        >
          G
        </span>
        Continuar con Google
      </Button>

      <div className="my-6 flex items-center gap-3 text-xs text-neutral-600">
        <span className="h-px flex-1 bg-neutral-800" />
        o con tu correo
        <span className="h-px flex-1 bg-neutral-800" />
      </div>

      <form onSubmit={(event) => void handleSubmit(event)} className="space-y-4" noValidate>
        {mode === "register" && (
          <div>
            <Label htmlFor="displayName">Nombre</Label>
            <Input
              id="displayName"
              name="displayName"
              autoComplete="name"
              placeholder="Cómo quieres aparecer"
              value={displayName}
              onChange={(event) => setDisplayName(event.target.value)}
            />
          </div>
        )}

        <div>
          <Label htmlFor="email">Correo</Label>
          <Input
            id="email"
            name="email"
            type="email"
            inputMode="email"
            autoComplete="email"
            placeholder="tu@correo.com"
            required
            value={email}
            onChange={(event) => setEmail(event.target.value)}
          />
        </div>

        <div>
          <Label htmlFor="password">Contraseña</Label>
          <Input
            id="password"
            name="password"
            type="password"
            autoComplete={mode === "login" ? "current-password" : "new-password"}
            placeholder="Mínimo 6 caracteres"
            required
            minLength={6}
            value={password}
            onChange={(event) => setPassword(event.target.value)}
          />
        </div>

        {error && <FieldError>{error}</FieldError>}

        <Button type="submit" className="w-full" disabled={busy}>
          {busy ? "Un momento…" : mode === "login" ? "Entrar" : "Crear cuenta"}
        </Button>
      </form>

      <p className="mt-6 text-center text-sm text-neutral-500">
        {mode === "login" ? "¿No tienes cuenta?" : "¿Ya tienes cuenta?"}{" "}
        <button
          type="button"
          className="text-neutral-200 underline-offset-4 hover:underline"
          onClick={() => switchMode(mode === "login" ? "register" : "login")}
        >
          {mode === "login" ? "Créala" : "Entra"}
        </button>
      </p>
    </Card>
  );
}

export default function AuthPanel() {
  const { user, ready } = useAuthUser();
  const { profile, state } = useApiProfile(user);

  if (!firebaseConfigured) {
    return (
      <div className="w-full max-w-md">
        <SetupNotice />
      </div>
    );
  }

  return (
    <div className="w-full max-w-md">
      {ready && user ? (
        <ProfileView ready={ready} user={user} profile={profile} syncState={state} />
      ) : (
        <AuthForm />
      )}
    </div>
  );
}
