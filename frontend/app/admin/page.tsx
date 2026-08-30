"use client";

import { OverviewCards } from "@/components/admin/OverviewCards";
import { useAsync } from "@/hooks/useAsync";
import { overview } from "@/services/admin";

export default function AdminOverviewPage() {
  const { data, error, loading, reload } = useAsync(overview, []);

  return (
    <div>
      {error ? <p className="mb-3 text-sm text-red-600">{error}</p> : null}
      {loading && !data ? (
        <p className="text-sm text-slate-500">Loading stats…</p>
      ) : (
        <OverviewCards stats={data} />
      )}
      <button
        type="button"
        onClick={reload}
        className="mt-4 rounded-lg border border-slate-300 px-3 py-1.5 text-xs font-medium text-slate-600 hover:bg-slate-50"
      >
        Refresh
      </button>
    </div>
  );
}
