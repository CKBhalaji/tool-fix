"use client";

import Link from "next/link";
import { Header } from "@/components/layout/Header";
import { Card } from "@/components/ui";

export default function AdminPage() {
  return (
    <>
      <Header title="Admin" />
      <main className="mx-auto w-full max-w-3xl flex-1 px-4 py-6">
        <Card>
          <h1 className="font-semibold">Admin console</h1>
          <p className="mt-1 text-sm text-slate-500">
            Role management, mechanic verification, and analytics ship in a later
            phase. The CUSTOMER/MECHANIC/ADMIN roles and authorization already
            exist in the backend (see backend/docs/AUTHENTICATION.md).
          </p>
          <div className="mt-4">
            <Link href="/" className="text-sm font-medium text-blue-600 hover:underline">
              Back to home
            </Link>
          </div>
        </Card>
      </main>
    </>
  );
}
