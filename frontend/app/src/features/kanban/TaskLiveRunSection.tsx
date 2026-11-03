import { useState, useEffect, useRef } from "react";
import { Square, Terminal, Loader2 } from "lucide-react";
import { t } from "../../core/i18n";
import { Button } from "../../ui/components/Button";
import { getSafeErrorMessage } from "../../core/api";

export interface EventItem {
  id?: string;
  seq: number;
  event_type: string;
  data: unknown;
  created_at?: string;
}

function extractChunkOrRaw(raw: string): string {
  try {
    const parsed = JSON.parse(raw);
    if (parsed && typeof parsed === "object") {
      if (typeof parsed.chunk === "string") return parsed.chunk;
      if (typeof parsed.message === "string") return parsed.message;
    }
  } catch {
    // raw string
  }
  return raw;
}

export function parseEventText(data: unknown): string {
  if (typeof data === "string") return extractChunkOrRaw(data);
  if (data && typeof data === "object") {
    const obj = data as Record<string, unknown>;
    if (typeof obj.chunk === "string") return obj.chunk;
    if (typeof obj.message === "string") return obj.message;
    return JSON.stringify(data);
  }
  return String(data ?? "");
}

interface Props {
  taskId: string;
  initialRunId?: string;
  onRunFinished?: () => void;
}

function useFindRunId(taskId: string, initialRunId?: string) {
  const [runId, setRunId] = useState<string | null>(initialRunId || null);

  useEffect(() => {
    if (runId) return;
    const req = fetch(`/api/v1/runs?task_id=${taskId}&limit=1`);
    if (!req || typeof req.then !== "function") return;
    req
      .then((res) => (res?.ok ? res.json() : null))
      .then((data) => {
        if (data?.runs?.[0]?.id) setRunId(data.runs[0].id);
      })
      .catch(() => {});
  }, [taskId, runId]);

  return { runId, setRunId };
}

function useLiveEvents(runId: string | null) {
  const [events, setEvents] = useState<EventItem[]>([]);

  useEffect(() => {
    if (!runId) return;
    const fetchEvents = async () => {
      try {
        const res = await fetch(`/api/v1/runs/${runId}/events?limit=100`);
        if (!res?.ok) return;
        const body = await res.json();
        if (Array.isArray(body?.events)) setEvents(body.events);
      } catch {
        // polling ignored
      }
    };
    fetchEvents();
    const timer = setInterval(fetchEvents, 1500);
    return () => clearInterval(timer);
  }, [runId]);

  return events;
}

function useCancelRun(runId: string | null, onRunFinished?: () => void) {
  const [cancelling, setCancelling] = useState(false);
  const [errorMsg, setErrorMsg] = useState<string | null>(null);
  const finishNotified = useRef(false);

  const handleCancel = async () => {
    if (!runId) return;
    setCancelling(true);
    setErrorMsg(null);
    try {
      const res = await fetch(`/api/v1/runs/${runId}/cancel`, { method: "POST" });
      if (!res?.ok) {
        const data = await res?.json?.().catch(() => ({})) ?? {};
        setErrorMsg(getSafeErrorMessage(data).userMessage);
      } else if (!finishNotified.current) {
        finishNotified.current = true;
        onRunFinished?.();
      }
    } catch (err) {
      setErrorMsg(getSafeErrorMessage(err).userMessage);
    } finally {
      setCancelling(false);
    }
  };

  return { cancelling, errorMsg, handleCancel };
}

export function TaskLiveRunSection({ taskId, initialRunId, onRunFinished }: Props) {
  const { runId } = useFindRunId(taskId, initialRunId);
  const events = useLiveEvents(runId);
  const { cancelling, errorMsg, handleCancel } = useCancelRun(runId, onRunFinished);

  return (
    <div style={{ display: "flex", flexDirection: "column", gap: "10px" }}>
      <LiveRunHeader cancelling={cancelling} onCancel={handleCancel} />
      {errorMsg && <div role="alert" style={{ color: "var(--rd)", fontSize: "12px" }}>{errorMsg}</div>}
      <LiveLogViewer events={events} />
    </div>
  );
}

function LiveRunHeader({ cancelling, onCancel }: { cancelling: boolean; onCancel: () => void }) {
  return (
    <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center" }}>
      <div style={{ display: "flex", alignItems: "center", gap: "6px", fontSize: "13px", fontWeight: 600 }}>
        <Loader2 size={14} className="animate-spin text-blue-500" />
        <span>{t("task_running_banner")}</span>
      </div>
      <Button
        variant="danger"
        size="sm"
        type="button"
        data-testid="btn-cancel-run"
        disabled={cancelling}
        onClick={onCancel}
        icon={<Square size={12} />}
      >
        {cancelling ? t("btn_cancelling") : t("btn_cancel_run")}
      </Button>
    </div>
  );
}

function LiveLogViewer({ events }: { events: EventItem[] }) {
  return (
    <div
      data-testid="task-live-logs"
      style={{
        background: "#0f172a",
        color: "#f8fafc",
        padding: "10px",
        borderRadius: "6px",
        fontFamily: "var(--font-mono)",
        fontSize: "12px",
        maxHeight: "220px",
        overflowY: "auto",
        display: "flex",
        flexDirection: "column",
        gap: "4px",
      }}
    >
      <div style={{ display: "flex", alignItems: "center", gap: "6px", color: "#94a3b8", fontSize: "11px", marginBottom: "4px" }}>
        <Terminal size={12} />
        <span>{t("task_live_log_title")}</span>
      </div>
      {events.length === 0 ? (
        <span style={{ color: "#64748b", fontStyle: "italic" }}>{t("task_live_log_empty")}</span>
      ) : (
        events.map((ev) => (
          <div key={ev.seq || ev.id} style={{ display: "flex", gap: "6px", wordBreak: "break-all" }}>
            <span style={{ color: "#64748b", userSelect: "none" }}>[{ev.seq}]</span>
            <span>{parseEventText(ev.data)}</span>
          </div>
        ))
      )}
    </div>
  );
}
