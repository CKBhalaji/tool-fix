"use client";

import { Card, EmptyState } from "@/components/ui";
import { formatMinor } from "@/types";
import type { MechanicPayment } from "@/services/mechanics";

/** The mechanic's incoming payments: amount, status, receipt, customer. */
export function EarningsPayments({ payments }: { payments: MechanicPayment[] }) {
  const confirmed = payments.filter((p) => p.status === "confirmed");
  const totalMinor = confirmed.reduce((sum, p) => sum + p.amount_minor, 0);

  return (
    <div className="grid gap-3">
      <Card>
        <p className="text-sm text-slate-500">Received (confirmed payments)</p>
        <p className="text-3xl font-bold text-slate-900">{formatMinor(totalMinor)}</p>
        <p className="mt-1 text-xs text-slate-500">{confirmed.length} confirmed payment(s)</p>
      </Card>

      {payments.length === 0 ? (
        <Card>
          <EmptyState message="No payments yet — they appear here after your first completed job." />
        </Card>
      ) : (
        <Card className="overflow-x-auto">
          <table className="w-full text-left text-sm">
            <thead>
              <tr className="text-xs uppercase text-slate-400">
                <th className="py-2 pr-3">Date</th>
                <th className="py-2 pr-3">Customer</th>
                <th className="py-2 pr-3">Amount</th>
                <th className="py-2 pr-3">Method</th>
                <th className="py-2 pr-3">Status</th>
                <th className="py-2">Receipt</th>
              </tr>
            </thead>
            <tbody>
              {payments.map((payment) => (
                <tr key={payment.payment_id} className="border-t border-slate-100">
                  <td className="py-2 pr-3 text-xs text-slate-500">
                    {new Date(payment.created_at).toLocaleString("en-IN")}
                  </td>
                  <td className="py-2 pr-3 text-xs text-slate-600">
                    {payment.customer_email ?? payment.customer_name ?? "—"}
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
                  <td className="py-2 text-xs text-slate-500">{payment.receipt_number ?? "—"}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </Card>
      )}
    </div>
  );
}
