import { Badge, Button, Card, CardDescription, CardTitle, FieldError } from "@tchat/ui";
import { useEffect, useState } from "react";

import { useAuthUser } from "../hooks/use-auth-user";
import { api, detailFrom } from "../lib/api";
import { firebaseConfigured, missingFirebaseEnv } from "../lib/firebase";
import { AuthForm } from "./AuthPanel";

function SetupNotice() {
  return (
    <Card>
      <Badge>Configuración pendiente</Badge>
      <CardTitle className="mt-4">Falta la configuración de Firebase</CardTitle>
      <CardDescription>
        Añade estas variables en <span className="font-mono">apps/account/.env</span> para poder
        autorizar el terminal.
      </CardDescription>
      <ul className="mt-4 space-y-1 font-mono text-xs text-neutral-400">
        {missingFirebaseEnv.map((name) => (
          <li key={name}>· {name}</li>
        ))}
      </ul>
    </Card>
  );
}

function approveMessage(code: string): string {
  switch (code) {
    case "invalid_code":
      return "Ese código no existe. Copia el que muestra tu terminal.";
    case "expired_code":
      return "El código caducó. Ejecuta de nuevo tchat login.";
    case "code_already_used":
      return "Ese código ya se utilizó. Genera uno nuevo en el terminal.";
    default:
      return "No se pudo autorizar el terminal. Inténtalo de nuevo.";
  }
}

export default function CliAuthPanel() {
  const { user, ready } = useAuthUser();
  const [code, setCode] = useState<string | null>(null);
  const [checkedQuery, setCheckedQuery] = useState(false);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [approved, setApproved] = useState(false);

  useEffect(() => {
    const value = new URLSearchParams(window.location.search).get("code");
    setCode(value ? value.trim().toUpperCase() : null);
    setCheckedQuery(true);
  }, []);

  async function approve() {
    if (!user || !code) {
      return;
    }

    setBusy(true);
    setError(null);

    try {
      const token = await user.getIdToken();
      const { error: requestError } = await api.POST("/api/v1/auth/device/approve", {
        headers: { Authorization: `Bearer ${token}` },
        body: { user_code: code },
      });

      if (requestError) {
        setError(approveMessage(detailFrom(requestError)));
        return;
      }

      setApproved(true);
    } catch {
      setError(approveMessage(""));
    } finally {
      setBusy(false);
    }
  }

  if (!firebaseConfigured) {
    return (
      <div className="w-full max-w-md">
        <SetupNotice />
      </div>
    );
  }

  if (!checkedQuery || !ready) {
    return (
      <div className="w-full max-w-md">
        <Card>
          <p className="text-sm text-neutral-500">Comprobando…</p>
        </Card>
      </div>
    );
  }

  if (!code) {
    return (
      <div className="w-full max-w-md">
        <Card>
          <Badge>Vinculación</Badge>
          <CardTitle className="mt-4">Abre esta página desde tu terminal</CardTitle>
          <CardDescription>
            Ejecuta el comando de login y TChat abrirá esta página con el código del dispositivo.
          </CardDescription>
          <pre className="mt-4 rounded-md border border-neutral-800 bg-neutral-950 p-3 font-mono text-sm text-neutral-200">
            <code>tchat login</code>
          </pre>
          <Button href="/" variant="secondary" className="mt-6 w-full">
            Ir a mi cuenta
          </Button>
        </Card>
      </div>
    );
  }

  if (!user) {
    return (
      <div className="w-full max-w-md">
        <AuthForm />
      </div>
    );
  }

  if (approved) {
    return (
      <div className="w-full max-w-md">
        <Card>
          <Badge>Autorizado</Badge>
          <CardTitle className="mt-4">Terminal autorizado</CardTitle>
          <CardDescription>
            Vuelve a tu terminal. La sesión debería abrirse en unos segundos.
          </CardDescription>
          <p className="mt-6 rounded-md border border-neutral-700 bg-neutral-950 py-4 text-center font-mono text-2xl tracking-[0.35em] text-neutral-100">
            {code}
          </p>
          <Button href="/" variant="secondary" className="mt-6 w-full">
            Ir a mi cuenta
          </Button>
        </Card>
      </div>
    );
  }

  return (
    <div className="w-full max-w-md">
      <Card>
        <Badge>Vinculación</Badge>
        <CardTitle className="mt-4">Autoriza este terminal</CardTitle>
        <CardDescription>
          Comprueba que el código coincide con el que muestra tu terminal antes de continuar.
        </CardDescription>

        <p className="mt-6 rounded-md border border-neutral-700 bg-neutral-950 py-4 text-center font-mono text-2xl tracking-[0.35em] text-neutral-100">
          {code}
        </p>

        <p className="mt-4 text-sm text-neutral-400">
          Cuenta: <span className="text-neutral-200">{user.email}</span>
        </p>

        {error && <FieldError>{error}</FieldError>}

        <Button className="mt-6 w-full" disabled={busy} onClick={() => void approve()}>
          {busy ? "Autorizando…" : "Autorizar terminal"}
        </Button>
        <p className="mt-3 text-center text-xs text-neutral-600">
          Si no reconoces este código, cierra esta página.
        </p>
      </Card>
    </div>
  );
}
