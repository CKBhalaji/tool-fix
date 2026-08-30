"use client";

import { Card, SecondaryButton } from "@/components/ui";
import type { MechanicProfile } from "@/types";

/** Service-area summary + editor. */
export function ServiceProfileCard({
  profile,
  onEditArea,
}: {
  profile: MechanicProfile | null;
  onEditArea: (areaKm: number) => void;
}) {
  function promptArea() {
    const value = window.prompt(
      "Service area radius in km",
      String(profile?.service_area_km ?? 10),
    );
    const parsed = Number(value);
    if (value && Number.isFinite(parsed)) {
      onEditArea(parsed);
    }
  }

  return (
    <Card>
      <h2 className="mb-2 font-semibold">Service profile</h2>
      <p className="text-xs text-slate-500">
        Service area: {profile?.service_area_km ?? 10} km around your shared location.
      </p>
      <div className="mt-3">
        <SecondaryButton onClick={promptArea}>Edit service area</SecondaryButton>
      </div>
    </Card>
  );
}
