"use client";

import { Card, EmptyState } from "@/components/ui";
import type { AdminUser } from "@/services/admin";

const ROLE_FILTERS = ["", "customer", "mechanic", "admin"] as const;

export function UsersTable({
  users,
  activeRole,
  onRoleChange,
  onToggleStatus,
}: {
  users: AdminUser[];
  activeRole: string;
  onRoleChange: (role: string) => void;
  onToggleStatus: (user: AdminUser) => void;
}) {
  return (
    <Card className="overflow-x-auto">
      <div className="mb-3 flex gap-2">
        {ROLE_FILTERS.map((value) => (
          <button
            key={value || "all"}
            type="button"
            onClick={() => onRoleChange(value)}
            className={`rounded-lg px-3 py-1.5 text-xs font-semibold ${
              activeRole === value ? "bg-slate-900 text-white" : "border border-slate-300 text-slate-600"
            }`}
          >
            {value || "All"}
          </button>
        ))}
      </div>
      <table className="w-full text-left text-sm">
        <thead>
          <tr className="text-xs uppercase text-slate-400">
            <th className="py-2 pr-3">Email / Name</th>
            <th className="py-2 pr-3">Role</th>
            <th className="py-2 pr-3">Status</th>
            <th className="py-2 pr-3">Joined</th>
            <th className="py-2">Actions</th>
          </tr>
        </thead>
        <tbody>
          {users.map((user) => (
            <tr key={user.id} className="border-t border-slate-100">
              <td className="py-2 pr-3">
                <p className="font-medium text-slate-800">{user.email ?? "—"}</p>
                <p className="text-xs text-slate-500">{user.display_name ?? ""}</p>
              </td>
              <td className="py-2 pr-3 capitalize">{user.role}</td>
              <td className="py-2 pr-3">{user.status}</td>
              <td className="py-2 pr-3 text-xs text-slate-500">
                {new Date(user.created_at).toLocaleDateString("en-IN")}
              </td>
              <td className="py-2">
                {user.role === "admin" ? (
                  <span className="text-xs text-slate-400">protected</span>
                ) : (
                  <button
                    type="button"
                    onClick={() => onToggleStatus(user)}
                    className="rounded-lg border border-slate-300 px-3 py-1 text-xs font-medium text-slate-600 hover:bg-slate-50"
                  >
                    {user.status === "active" ? "Suspend" : "Activate"}
                  </button>
                )}
              </td>
            </tr>
          ))}
        </tbody>
      </table>
      <EmptyState message={users.length === 0 ? "No users." : ""} />
    </Card>
  );
}
