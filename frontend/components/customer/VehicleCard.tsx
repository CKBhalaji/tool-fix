"use client";

import { Card } from "@/components/ui";
import type { Vehicle } from "@/types";

export function VehicleCard({ vehicle }: { vehicle: Vehicle }) {
  return (
    <Card>
      <p className="font-semibold capitalize">{vehicle.vehicle_kind}</p>
      <p className="text-xs text-muted">
        {[vehicle.make, vehicle.model, vehicle.year].filter(Boolean).join(" ") || "—"}
      </p>
    </Card>
  );
}
