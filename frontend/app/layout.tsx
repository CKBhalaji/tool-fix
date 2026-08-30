import type { Metadata } from "next";
import { AuthProvider } from "@/hooks/useAuth";
import "./globals.css";

export const metadata: Metadata = {
  title: "ToolFix — On-demand roadside assistance",
  description:
    "Stranded with a breakdown? ToolFix analyzes your problem, notifies nearby mechanics, and lets them bid to help you.",
};

export default function RootLayout({ children }: LayoutProps<"/">) {
  return (
    <html lang="en" className="h-full antialiased">
      <body
        suppressHydrationWarning
        className="min-h-full flex flex-col bg-slate-50 text-slate-900"
      >
        <AuthProvider>{children}</AuthProvider>
      </body>
    </html>
  );
}
