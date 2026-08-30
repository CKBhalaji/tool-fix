"use client";

import { Card, StatCard } from "@/components/ui";
import { formatMinor } from "@/types";
import type { AdminOverview } from "@/services/admin";

export function OverviewCards({ stats }: { stats: AdminOverview | null }) {
  if (!stats) {
    return <Card><p className="text-sm text-muted">Loading stats…</p></Card>;
  }

  const cards: { label: string; value: string | number }[] = [
    { label: "Total users", value: stats.users_total },
    { label: "Customers", value: stats.customers },
    { label: "Mechanics", value: `${stats.mechanics} (${stats.mechanics_verified} verified)` },
    { label: "Admins", value: stats.admins },
    { label: "Jobs", value: stats.jobs_total },
    { label: "Active jobs", value: stats.jobs_active },
    { label: "Completed jobs", value: stats.jobs_completed },
    { label: "Offers", value: stats.offers_total },
    { label: "Payments collected", value: formatMinor(stats.payments_collected_minor) },
    { label: "Payments", value: `${stats.payments_count} confirmed` },
    { label: "AI runs", value: stats.ai_runs_total },
  ];

  return (
    <div className="grid gap-3 sm:grid-cols-2 lg:grid-cols-4">
      {cards.map((card) => (
        <StatCard key={card.label} label={card.label} value={card.value} />
      ))}
    </div>
  );
}
