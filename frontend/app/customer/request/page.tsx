"use client";

import { useRouter } from "next/navigation";
import { useAuth } from "@/hooks/useAuth";
import { Header } from "@/components/layout/Header";
import { BreakdownRequestForm } from "@/components/customer/BreakdownRequestForm";

export default function RequestAssistancePage() {
  const { me, loading } = useAuth();
  const router = useRouter();

  if (!loading && (!me || me.user.role !== "customer")) {
    router.replace("/");
  }

  return (
    <>
      <Header title="Request assistance" />
      <main className="mx-auto w-full max-w-3xl flex-1 px-4 py-6">
        <BreakdownRequestForm />
      </main>
    </>
  );
}
