"use client";

import Link from "next/link";
import { useAuth } from "@/hooks/useAuth";

export function Header({ title }: { title?: string }) {
  const { me, logout } = useAuth();

  return (
    <header className="border-b border-slate-200 bg-white">
      <div className="mx-auto flex max-w-5xl items-center justify-between px-4 py-3">
        <Link href="/" className="flex items-center gap-2 font-bold text-slate-900">
          <span className="grid h-7 w-7 place-items-center rounded-lg bg-blue-600 text-white">T</span>
          ToolFix
        </Link>
        {title ? <p className="text-sm font-medium text-slate-500">{title}</p> : null}
        {me ? (
          <div className="flex items-center gap-3 text-sm">
            <span className="hidden text-slate-500 sm:inline">
              {me.user.email ?? me.user.display_name ?? "Signed in"}
            </span>
            <button
              type="button"
              onClick={() => void logout()}
              className="rounded-lg border border-slate-300 px-3 py-1.5 font-medium text-slate-600 hover:bg-slate-50"
            >
              Sign out
            </button>
          </div>
        ) : null}
      </div>
    </header>
  );
}
