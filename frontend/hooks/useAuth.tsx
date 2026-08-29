"use client";

import { createContext, useCallback, useContext, useEffect, useState } from "react";
import { getCurrentUser, logout as authLogout } from "@/services/auth";
import type { AuthMe, UserRole } from "@/types";

interface AuthState {
  me: AuthMe | null;
  loading: boolean;
  refresh: () => Promise<void>;
  logout: () => Promise<void>;
  switchRole: (role: UserRole) => Promise<void>;
}

const AuthContext = createContext<AuthState>({
  me: null,
  loading: true,
  refresh: async () => {},
  logout: async () => {},
  switchRole: async () => {},
});

export function AuthProvider({ children }: { children: React.ReactNode }) {
  const [me, setMe] = useState<AuthMe | null>(null);
  const [loading, setLoading] = useState(true);

  const refresh = useCallback(async () => {
    try {
      const result = await getCurrentUser();
      setMe(result);
    } catch {
      setMe(null);
    } finally {
      setLoading(false);
    }
  }, []);

  const logout = useCallback(async () => {
    await authLogout();
    setMe(null);
    routerPush("/login");
  }, []);

  const switchRole = useCallback(
    async (role: UserRole) => {
      const { completeOnboarding } = await import("@/services/auth");
      await completeOnboarding(role);
      await refresh();
    },
    [refresh],
  );

  // Initial session probe: setState happens in promise continuations.
  useEffect(() => {
    let ignore = false;
    getCurrentUser()
      .then((result) => {
        if (!ignore) setMe(result);
      })
      .catch(() => {
        if (!ignore) setMe(null);
      })
      .finally(() => {
        if (!ignore) setLoading(false);
      });
    return () => {
      ignore = true;
    };
  }, []);

  return (
    <AuthContext.Provider value={{ me, loading, refresh, logout, switchRole }}>
      {children}
    </AuthContext.Provider>
  );
}

function routerPush(path: string) {
  window.location.assign(path);
}

export function useAuth(): AuthState {
  return useContext(AuthContext);
}
