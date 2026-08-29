import { api } from "./api";
import type { Job, JobEvent, JobStatusHistoryEntry } from "@/types";

export async function getJob(id: string): Promise<Job> {
  return api<Job>(`/api/v1/jobs/${id}`);
}

export async function listMyJobs(): Promise<Job[]> {
  return api<Job[]>("/api/v1/jobs");
}

export async function getStatusHistory(id: string): Promise<JobStatusHistoryEntry[]> {
  return api<JobStatusHistoryEntry[]>(`/api/v1/jobs/${id}/status-history`);
}

/** Mechanic execution path (state-machine guarded on the server). */
export async function startTravel(jobId: string): Promise<Job> {
  return api<Job>(`/api/v1/jobs/${jobId}/start-travel`, { method: "POST" });
}

export async function markArrived(jobId: string): Promise<Job> {
  return api<Job>(`/api/v1/jobs/${jobId}/arrived`, { method: "POST" });
}

export async function startRepair(jobId: string): Promise<Job> {
  return api<Job>(`/api/v1/jobs/${jobId}/start-repair`, { method: "POST" });
}

export async function completeRepair(jobId: string): Promise<Job> {
  return api<Job>(`/api/v1/jobs/${jobId}/complete`, { method: "POST" });
}

/** Customer pays the accepted offer amount (stub/cash provider). */
export async function payJob(jobId: string): Promise<unknown> {
  return api(`/api/v1/jobs/${jobId}/pay`, {
    method: "POST",
    body: { method: "cash" },
  });
}

/** Subscribes to a job's live event channel. Returns a disposer. */
export function subscribeJobEvents(
  jobId: string,
  onEvent: (event: JobEvent) => void,
): () => void {
  const wsBase = process.env.NEXT_PUBLIC_WS_URL ?? "ws://localhost:8080";
  let socket: WebSocket | null = null;
  let closed = false;
  let retry: ReturnType<typeof setTimeout> | null = null;

  function connect() {
    if (closed) return;
    socket = new WebSocket(`${wsBase}/api/v1/ws/jobs/${jobId}`);
    socket.onmessage = (message) => {
      try {
        onEvent(JSON.parse(message.data) as JobEvent);
      } catch {
        // ignore malformed frames; REST remains the source of truth
      }
    };
    socket.onclose = () => {
      if (!closed) {
        retry = setTimeout(connect, 2000);
      }
    };
  }

  connect();
  return () => {
    closed = true;
    if (retry) clearTimeout(retry);
    socket?.close();
  };
}
