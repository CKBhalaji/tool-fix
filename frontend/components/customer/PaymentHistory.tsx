"use client";

import { Card, EmptyState } from "@/components/ui";
import { formatMinor } from "@/types";

export interface CustomerPayment {
  payment_id: string;
  job_id: string;
  amount_minor: number;
  method: string;
  status: string;
  receipt_number: string | null;
  created_at: string;
  confirmed_at: string | null;
  job_status: string;
}

/** The customer's own payment history. */
export function PaymentHistory({ payments }: { payments: CustomerPayment[] }) {
  if (payments.length === 0) {
    return (
      <Card>
        <EmptyState message="No payments yet — they appear here after your first completed repair." />
      </Card>
    );
  }

  return (
    <div className="grid gap-3">
      {payments.map((payment) => (
        <Card key={payment.payment_id} className="flex flex-wrap items-center justify-between gap-3">
          <div>
            <p className="font-semibold text-foreground">{formatMinor(payment.amount_minor)}</p>
            <p className="text-xs text-muted">
              {new Date(payment.created_at).toLocaleString("en-IN")} · {payment.method}
            </p>
          </div>
          <span
            className={`inline-block rounded-full px-2.5 py-0.5 text-xs font-medium ${
              payment.status === "confirmed"
                ? "bg-success-light text-success"
                : payment.status === "failed"
                  ? "bg-error-light text-error"
                  : "bg-warning-light text-warning"
            }`}
          >
            {payment.status}
          </span>
          <p className="text-xs text-muted">Receipt: {payment.receipt_number ?? "—"}</p>
        </Card>
      ))}
    </div>
  );
}
