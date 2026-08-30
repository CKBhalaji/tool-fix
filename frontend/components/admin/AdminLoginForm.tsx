"use client";

import { useState } from "react";
import { Card, ErrorText, PrimaryButton, TextInput } from "@/components/ui";
import { adminLogin } from "@/services/admin";

/**
 * Static-credential admin sign-in. On success the backend sets the same
 * HttpOnly session cookies (role forced to ADMIN server-side).
 */
export function AdminLoginForm({ onSignedIn }: { onSignedIn: () => void }) {
  const [email, setEmail] = useState("admin@toolfix.com");
  const [password, setPassword] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [signingIn, setSigningIn] = useState(false);

  async function signIn() {
    setError(null);
    setSigningIn(true);
    try {
      await adminLogin(email, password);
      onSignedIn();
    } catch (err) {
      setError(err instanceof Error ? err.message : "Login failed");
    } finally {
      setSigningIn(false);
    }
  }

  return (
    <Card className="w-full max-w-sm">
      <h1 className="text-lg font-bold text-slate-900">Admin sign in</h1>
      <p className="mt-1 text-xs text-slate-500">
        Operator access only. Sessions are HttpOnly cookies; all actions are audit-logged.
      </p>
      <div className="mt-4 grid gap-3">
        <TextInput value={email} onChange={setEmail} placeholder="admin@toolfix.com" />
        <TextInput type="password" value={password} onChange={setPassword} placeholder="Password" />
        <ErrorText>{error}</ErrorText>
        <PrimaryButton onClick={signIn} disabled={signingIn} className="w-full">
          {signingIn ? "Signing in…" : "Sign in"}
        </PrimaryButton>
      </div>
    </Card>
  );
}
