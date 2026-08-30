"use client";

import Link from "next/link";
import { usePathname } from "next/navigation";
import { Header } from "@/components/layout/Header";
import { AdminLoginForm } from "@/components/admin/AdminLoginForm";
import { useAuth } from "@/hooks/useAuth";

const NAV = [
  { href: "/admin", label: "Overview" },
  { href: "/admin/users", label: "Users" },
  { href: "/admin/mechanics", label: "Mechanics" },
  { href: "/admin/jobs", label: "Jobs" },
  { href: "/admin/payments", label: "Payments" },
  { href: "/admin/ai", label: "AI Usage" },
];

/**
 * Admin layout: gates every /admin route behind the ADMIN session (decided
 * server-side) and renders the section navigation once.
 */
export default function AdminLayout({ children }: { children: React.ReactNode }) {
  const { me, loading, refresh } = useAuth();
  const pathname = usePathname();

  if (loading) {
    return (
      <>
        <Header title="Admin console" />
        <main className="flex flex-1 items-center justify-center text-sm text-slate-500">Loading…</main>
      </>
    );
  }

  if (me?.user.role !== "admin") {
    return (
      <>
        <Header title="Admin console" />
        <main className="flex flex-1 items-center justify-center px-4">
          <AdminLoginForm onSignedIn={() => void refresh()} />
        </main>
      </>
    );
  }

  return (
    <>
      <Header title="Admin console" />
      <div className="mx-auto w-full max-w-6xl px-4 pt-6">
        <nav className="mb-5 flex gap-2">
          {NAV.map((item) => {
            const active = pathname === item.href;
            return (
              <Link
                key={item.href}
                href={item.href}
                className={`rounded-xl px-4 py-2 text-sm font-semibold capitalize transition ${
                  active
                    ? "bg-slate-900 text-white"
                    : "border border-slate-300 bg-white text-slate-600 hover:bg-slate-50"
                }`}
              >
                {item.label}
              </Link>
            );
          })}
        </nav>
      </div>
      <main className="mx-auto w-full max-w-6xl flex-1 px-4 pb-6">{children}</main>
    </>
  );
}
