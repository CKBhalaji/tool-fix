"use client";

import Link from "next/link";
import { usePathname } from "next/navigation";
import { useAuth } from "@/hooks/useAuth";

interface NavItem {
  href: string;
  label: string;
  icon: React.ReactNode;
}

function Icon({ path }: { path: string }) {
  return (
    <svg width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.8" strokeLinecap="round" strokeLinejoin="round" aria-hidden="true">
      <path d={path} />
    </svg>
  );
}

const CUSTOMER_ITEMS: NavItem[] = [
  { href: "/customer/dashboard", label: "Home", icon: <Icon path="M3 10.5 12 3l9 7.5M5 9.5V21h14V9.5" /> },
  { href: "/customer/request", label: "Request", icon: <Icon path="M12 5v14M5 12h14" /> },
  { href: "/customer/payments", label: "Payments", icon: <Icon path="M2 8h20M2 8v10a2 2 0 0 0 2 2h16a2 2 0 0 0 2-2V8M2 8V6a2 2 0 0 1 2-2h16a2 2 0 0 1 2 2v2M6 14h4" /> },
  { href: "/customer/history", label: "History", icon: <Icon path="M3 12a9 9 0 1 0 3-6.7L3 8m0-5v5h5M12 7v5l3 3" /> },
];

const MECHANIC_ITEMS: NavItem[] = [
  { href: "/mechanic/dashboard", label: "Home", icon: <Icon path="M3 10.5 12 3l9 7.5M5 9.5V21h14V9.5" /> },
  { href: "/mechanic/requests", label: "Requests", icon: <Icon path="M12 5v14M5 12h14" /> },
  { href: "/mechanic/jobs", label: "Jobs", icon: <Icon path="M14.7 6.3a4 4 0 0 0-5.4 5.4L3 18v3h3l6.3-6.3a4 4 0 0 0 5.4-5.4l-2.6 2.6-2.1-2.1 2.7-2.9Z" /> },
  { href: "/mechanic/payments", label: "Payments", icon: <Icon path="M2 8h20M2 8v10a2 2 0 0 0 2 2h16a2 2 0 0 0 2-2V8M2 8V6a2 2 0 0 1 2-2h16a2 2 0 0 1 2 2v2M6 14h4" /> },
];

const ADMIN_ITEMS: NavItem[] = [
  { href: "/admin", label: "Overview", icon: <Icon path="M3 12h7V3H3v9Zm11 9h7v-9h-7v9ZM3 21h7v-5H3v5Zm11-13h7V3h-7v5Z" /> },
  { href: "/admin/users", label: "Users", icon: <Icon path="M16 21v-2a4 4 0 0 0-4-4H6a4 4 0 0 0-4 4v2M9 11a4 4 0 1 0 0-8 4 4 0 0 0 0 8Zm13 10v-2a4 4 0 0 0-3-3.9M16 3.1a4 4 0 0 1 0 7.8" /> },
  { href: "/admin/jobs", label: "Jobs", icon: <Icon path="M14.7 6.3a4 4 0 0 0-5.4 5.4L3 18v3h3l6.3-6.3a4 4 0 0 0 5.4-5.4l-2.6 2.6-2.1-2.1 2.7-2.9Z" /> },
  { href: "/admin/payments", label: "Payments", icon: <Icon path="M2 8h20M2 8v10a2 2 0 0 0 2 2h16a2 2 0 0 0 2-2V8M2 8V6a2 2 0 0 1 2-2h16a2 2 0 0 1 2 2v2M6 14h4" /> },
];

function itemsForRole(role: string | undefined): NavItem[] {
  if (role === "mechanic") return MECHANIC_ITEMS;
  if (role === "admin") return ADMIN_ITEMS;
  return CUSTOMER_ITEMS;
}

/**
 * Mobile bottom navigation (role-aware). Rendered by the Header; hidden on
 * md+ where the top navigation takes over. Touch targets >= 44px.
 */
export function MobileNav() {
  const { me } = useAuth();
  const pathname = usePathname();
  const items = itemsForRole(me?.user.role);

  return (
    <>
      {/* Spacer so fixed bar never covers page content */}
      <div className="h-16 md:hidden" aria-hidden="true" />
      <nav
        aria-label="Primary"
        className="fixed inset-x-0 bottom-0 z-40 border-t border-border bg-surface md:hidden"
        style={{ paddingBottom: "env(safe-area-inset-bottom)" }}
      >
        <div className="mx-auto grid max-w-lg grid-cols-4">
          {items.map((item) => {
            const active = pathname === item.href;
            return (
              <Link
                key={item.href}
                href={item.href}
                aria-current={active ? "page" : undefined}
                className={`flex min-h-[56px] flex-col items-center justify-center gap-0.5 text-[11px] font-medium transition ${
                  active ? "text-primary" : "text-muted"
                }`}
              >
                {item.icon}
                {item.label}
              </Link>
            );
          })}
        </div>
      </nav>
    </>
  );
}
