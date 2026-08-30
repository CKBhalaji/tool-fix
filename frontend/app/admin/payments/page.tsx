"use client";

import { PaymentsTable } from "@/components/admin/PaymentsTable";
import { useAsync } from "@/hooks/useAsync";
import { listPayments } from "@/services/admin";

export default function AdminPaymentsPage() {
  const { data, error } = useAsync(listPayments, []);

  return (
    <div>
      {error ? <p className="mb-3 text-sm text-red-600">{error}</p> : null}
      <PaymentsTable payments={data ?? []} />
    </div>
  );
}
