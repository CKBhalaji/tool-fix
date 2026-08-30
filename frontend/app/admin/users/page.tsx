"use client";

import { useState } from "react";
import { UsersTable } from "@/components/admin/UsersTable";
import { useAsync } from "@/hooks/useAsync";
import { listUsers, setUserStatus } from "@/services/admin";
import type { AdminUser } from "@/services/admin";
import type { UserStatus } from "@/types";

export default function AdminUsersPage() {
  const [role, setRole] = useState("");
  const { data, error, reload } = useAsync(() => listUsers(role || undefined), [role]);

  async function toggleStatus(user: AdminUser) {
    const next: UserStatus = user.status === "active" ? "suspended" : "active";
    await setUserStatus(user.id, next);
    reload();
  }

  return (
    <div>
      {error ? <p className="mb-3 text-sm text-red-600">{error}</p> : null}
      <UsersTable
        users={data ?? []}
        activeRole={role}
        onRoleChange={setRole}
        onToggleStatus={(user) => void toggleStatus(user)}
      />
    </div>
  );
}
