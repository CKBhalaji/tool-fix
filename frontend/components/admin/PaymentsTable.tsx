"use client";

import { Card, EmptyState } from "@/components/ui";
import { formatMinor } from "@/types";
import type { AdminPayment } from "@/services/admin";

/** Every payment in the platform (admin view). */
export function PaymentsTable({ payments }: { payments: AdminPayment[] }) {
  return (
    <Card className="overflow-x-auto">
      <table className="w-full text-left text-sm">
        <thead>
          <tr className="text-xs uppercase text-slate-400">
            <th className="py-2 pr-3">Created</th>
            <th className="py-2 pr-3">Amount</th>
            <th className="py-2 pr-3">Method</th>
            <th className="py-2 pr-3">Status</th>
            <th className="py-2 pr-3">Receipt</th>
            <th className="py-2 pr-3">Customer</th>
            <th className="py-2">Job status</th>
          </tr>
        </thead>
        <tbody>
          {payments.map((payment) => (
            <tr key={payment.payment_id} className="border-t border-slate-100">
              <td className="py-2 pr-3 text-xs text-slate-500">
                {new Date(payment.created_at).toLocaleString("en-IN")}
              </td>
              <td className="py-2 pr-3 font-semibold">{formatMinor(payment.amount_minor)}</td>
              <td className="py-2 pr-3 capitalize">{payment.method}</td>
              <td className="py-2 pr-3">
                <span
                  className={`inline-block rounded-full px-2.5 py-0.5 text-xs font-medium ${
                    payment.status === "confirmed"
                      ? "bg-green-100 text-green-700"
                      : payment.status === "failed"
                        ? "bg-red-100 text-red-700"
                        : "bg-yellow-100 text-yellow-800"
                  }`}
                >
                  {payment.status}
                </span>
              </td>
              <td className="py-2 pr-3 text-xs text-slate-500">{payment.receipt_number ?? "—"}</td>
              <td className="py-2 pr-3 text-xs text-slate-600">
                {payment.customer_email ?? payment.customer_name ?? "—"}
              </td>
              <td className="py-2 pr-3 text-xs text-slate-500 capitalize">
                {payment.job_status.replace(/_/g, " ")}
              </td>
            </tr>
          ))}
        </tbody>
      </table>
      <EmptyState message={payments.length === 0 ? "No payments yet." : ""} />
    </Card>
  );
}
