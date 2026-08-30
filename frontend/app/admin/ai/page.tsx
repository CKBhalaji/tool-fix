"use client";

import { AgentRunsTable } from "@/components/admin/AgentRunsTable";
import { useAsync } from "@/hooks/useAsync";
import { listAgentRuns } from "@/services/admin";

export default function AdminAiUsagePage() {
  const { data, error } = useAsync(listAgentRuns, []);

  return (
    <div>
      {error ? <p className="mb-3 text-sm text-red-600">{error}</p> : null}
      <AgentRunsTable runs={data ?? []} />
    </div>
  );
}
