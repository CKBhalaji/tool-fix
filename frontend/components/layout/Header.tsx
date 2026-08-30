"use client";

import Image from "next/image";
import Link from "next/link";
import { useAuth } from "@/hooks/useAuth";
import { ThemeToggle } from "@/components/theme/ThemeToggle";
import { MobileNav } from "@/components/layout/MobileNav";

/**
 * Top navigation: logo (light/dark variants), role links (desktop), theme
 * toggle, and sign-out. Mobile gets the role-aware bottom nav.
 */
export function Header({ title }: { title?: string }) {
  const { me, logout } = useAuth();

  return (
    <>
      <header className="border-b border-border bg-surface">
        <div className="mx-auto flex max-w-6xl items-center justify-between gap-3 px-4 py-3">
          <Link href="/" className="flex items-center gap-2" aria-label="ToolFix home">
            <Image
              src="/brand/logo-light.png"
              alt="ToolFix"
              width={120}
              height={32}
              priority
              className="h-8 w-auto dark:hidden"
            />
            <Image
              src="/brand/logo-dark.png"
              alt="ToolFix"
              width={120}
              height={32}
              priority
              className="hidden h-8 w-auto dark:block"
            />
          </Link>
          {title ? (
            <p className="hidden text-sm font-medium text-secondary sm:block">{title}</p>
          ) : null}
          <div className="flex items-center gap-2 text-sm">
            <ThemeToggle />
            {me ? (
              <>
                <span className="hidden max-w-[10rem] truncate text-muted sm:inline">
                  {me.user.email ?? me.user.display_name ?? "Signed in"}
                </span>
                <button
                  type="button"
                  onClick={() => void logout()}
                  className="rounded-lg border border-border px-3 py-1.5 font-medium text-secondary transition hover:bg-surface-secondary"
                >
                  Sign out
                </button>
              </>
            ) : null}
          </div>
        </div>
      </header>
      <MobileNav />
    </>
  );
}
