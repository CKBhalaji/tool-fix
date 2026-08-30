"use client";

import { Card, PrimaryButton } from "@/components/ui";
import type { MechanicProfile } from "@/types";

/** Online/offline toggle with rating + verification summary. */
export function AvailabilityPanel({
  profile,
  busy,
  onToggle,
}: {
  profile: MechanicProfile | null;
  busy: boolean;
  onToggle: () => void;
}) {
  const online = profile?.availability_status !== "offline";

  return (
    <Card>
      <div className="flex flex-wrap items-center justify-between gap-4">
        <div>
          <p className="text-sm font-semibold text-foreground">
            {profile?.display_name ?? "Mechanic"}
          </p>
          <p className="text-xs text-muted">
            {profile?.completed_jobs ?? 0} completed jobs ·{" "}
            {profile?.rating_average ? `★ ${profile.rating_average.toFixed(1)}` : "not yet rated"}
            {profile?.is_verified ? "" : " · verification pending"}
          </p>
        </div>
        <div className="flex items-center gap-3">
          <span
            className={`inline-block rounded-full px-3 py-1 text-xs font-semibold ${
              online ? "bg-success-light text-success" : "bg-surface-secondary text-secondary"
            }`}
          >
            {profile?.availability_status ?? "offline"}
          </span>
          <PrimaryButton onClick={onToggle} disabled={busy || !profile}>
            {online ? "Go offline" : "Go online"}
          </PrimaryButton>
        </div>
      </div>
      {profile?.repair_categories.length ? (
        <p className="mt-3 text-xs text-muted">
          Services: {profile.repair_categories.join(", ")}
        </p>
      ) : (
        <p className="mt-3 text-xs text-warning">
          Set your services in your profile to appear in matching.
        </p>
      )}
    </Card>
  );
}
