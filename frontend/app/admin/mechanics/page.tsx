"use client";

import { MechanicsTable } from "@/components/admin/MechanicsTable";
import { useAsync } from "@/hooks/useAsync";
import { listMechanics, setVerified } from "@/services/admin";
import type { AdminMechanic } from "@/services/admin";

export default function AdminMechanicsPage() {
  const { data, error, reload } = useAsync(listMechanics, []);

  async function toggleVerified(mechanic: AdminMechanic) {
    await setVerified(mechanic.mechanic_id, !mechanic.is_verified);
    reload();
  }

  return (
    <div>
      {error ? <p className="mb-3 text-sm text-red-600">{error}</p> : null}
      <MechanicsTable
        mechanics={data ?? []}
        onToggleVerified={(mechanic) => void toggleVerified(mechanic)}
      />
    </div>
  );
}
