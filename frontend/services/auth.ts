// Application-level auth service. There is NO Firebase here and no token
// handling: the backend performs the Google/Firebase flow server-side and
// sets HttpOnly cookies. The button simply navigates to the backend's
// consent URL.

import { api } from "./api";
import { apiPath } from "@/lib/config";
import type { AuthMe, User, UserRole } from "@/types";

/** "Continue with Google" — full-page redirect to the backend OAuth entry. */
export function continueWithGoogle(): void {
  window.location.href = apiPath("/api/v1/auth/google/login");
}

export async function getCurrentUser(): Promise<AuthMe> {
  return api<AuthMe>("/api/v1/auth/me", { skipRefresh: true });
}

export async function logout(): Promise<void> {
  await api("/api/v1/auth/logout", { method: "POST", skipRefresh: true });
}

export async function completeOnboarding(
  requestedRole: UserRole,
  phone?: string,
  displayName?: string,
): Promise<User> {
  return api<User>("/api/v1/users/onboarding", {
    method: "POST",
    body: {
      requested_role: requestedRole,
      phone: phone ?? null,
      display_name: displayName ?? null,
    },
  });
}
