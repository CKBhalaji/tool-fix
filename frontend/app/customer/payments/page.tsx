"use client";

import { useAuth } from "@/hooks/useAuth";
import { useAsync } from "@/hooks/useAsync";
import { Header } from "@/components/layout/Header";
import { PaymentHistory } from "@/components/customer/PaymentHistory";
import { api } from "@/services/api";

interface Payment {
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

export default function CustomerPaymentsPage() {
  const { me } = useAuth();
  const payments = useAsync(() => api<Payment[]>("/api/v1/payments"), [me?.user.id]);

  return (
    <>
      <Header title="My payments" />
      <main className="mx-auto w-full max-w-3xl flex-1 px-4 py-6">
        {payments.error ? (
          <p className="mb-3 text-sm text-red-600">{payments.error}</p>
        ) : null}
        <PaymentHistory payments={payments.data ?? []} />
      </main>
    </>
  );
}
