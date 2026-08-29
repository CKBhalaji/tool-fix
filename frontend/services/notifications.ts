import { api } from "./api";
import type { AppNotification } from "@/types";

export async function listNotifications(): Promise<AppNotification[]> {
  return api<AppNotification[]>("/api/v1/notifications");
}
