"use client";

import { Card, EmptyState } from "@/components/ui";
import type { AdminMechanic } from "@/services/admin";

export function MechanicsTable({
  mechanics,
  onToggleVerified,
}: {
  mechanics: AdminMechanic[];
  onToggleVerified: (mechanic: AdminMechanic) => void;
}) {
  return (
    <Card className="overflow-x-auto">
      <table className="w-full text-left text-sm">
        <thead>
          <tr className="text-xs uppercase text-slate-400">
            <th className="py-2 pr-3">Mechanic</th>
            <th className="py-2 pr-3">City / Area</th>
            <th className="py-2 pr-3">Availability</th>
            <th className="py-2 pr-3">Rating</th>
            <th className="py-2 pr-3">Jobs</th>
            <th className="py-2">Verified</th>
          </tr>
        </thead>
        <tbody>
          {mechanics.map((mechanic) => (
            <tr key={mechanic.mechanic_id} className="border-t border-slate-100">
              <td className="py-2 pr-3">
                <p className="font-medium text-slate-800">{mechanic.display_name ?? "—"}</p>
                <p className="text-xs text-slate-500">{mechanic.user.email ?? ""}</p>
              </td>
              <td className="py-2 pr-3 text-xs text-slate-600">
                {mechanic.city ?? "—"} · {mechanic.service_area_km} km
              </td>
              <td className="py-2 pr-3 capitalize">{mechanic.availability_status}</td>
              <td className="py-2 pr-3">
                {mechanic.rating_average ? `★ ${mechanic.rating_average.toFixed(1)}` : "—"}
              </td>
              <td className="py-2 pr-3">{mechanic.completed_jobs}</td>
              <td className="py-2">
                <button
                  type="button"
                  onClick={() => onToggleVerified(mechanic)}
                  className={`rounded-lg px-3 py-1 text-xs font-semibold ${
                    mechanic.is_verified
                      ? "bg-green-100 text-green-700"
                      : "border border-slate-300 text-slate-600"
                  }`}
                >
                  {mechanic.is_verified ? "Verified" : "Verify"}
                </button>
              </td>
            </tr>
          ))}
        </tbody>
      </table>
      <EmptyState message={mechanics.length === 0 ? "No mechanics." : ""} />
    </Card>
  );
}
