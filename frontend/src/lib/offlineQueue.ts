export interface QueuedMutation {
  url: string;
  method: string;
  headers: Record<string, string>;
  body?: string;
  timestamp: number;
}

export function queueMutation(mutation: Omit<QueuedMutation, "timestamp">) {
  try {
    const queue = localStorage.getItem("storeit_offline_queue");
    const items: QueuedMutation[] = queue ? JSON.parse(queue) : [];
    items.push({ ...mutation, timestamp: Date.now() });
    localStorage.setItem("storeit_offline_queue", JSON.stringify(items));
    window.dispatchEvent(new CustomEvent("storeit:queue-updated"));
  } catch {
    // storage full or unavailable — silently fail
  }
}

export function getPendingQueue(): QueuedMutation[] {
  try {
    const queue = localStorage.getItem("storeit_offline_queue");
    return queue ? JSON.parse(queue) : [];
  } catch {
    return [];
  }
}

export async function flushPendingQueue(): Promise<number> {
  try {
    const queue = localStorage.getItem("storeit_offline_queue");
    if (!queue) return 0;
    const items: QueuedMutation[] = JSON.parse(queue);
    if (items.length === 0) return 0;

    const remaining: QueuedMutation[] = [];
    for (const item of items) {
      try {
        const res = await fetch(item.url, {
          method: item.method,
          headers: item.headers,
          body: item.body,
          credentials: "same-origin",
        });
        if (!res.ok && res.status !== 401) {
          remaining.push(item);
        }
      } catch {
        remaining.push(item);
      }
    }

    if (remaining.length > 0) {
      localStorage.setItem("storeit_offline_queue", JSON.stringify(remaining));
    } else {
      localStorage.removeItem("storeit_offline_queue");
    }
    return remaining.length;
  } catch {
    return 0;
  }
}
