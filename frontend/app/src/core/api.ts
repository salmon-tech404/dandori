import { z } from "zod";
import { t } from "./i18n";

export interface ApiErrorPayload {
  error: {
    code: string;
    message: string;
    request_id: string;
  };
}

export const ApiErrorSchema = z.strictObject({
  error: z.strictObject({
    code: z.string(),
    message: z.string(),
    request_id: z.string(),
  }),
});

export class DandoriApiError extends Error {
  public readonly code: string;
  public readonly requestId: string;

  constructor(code: string, message: string, requestId: string) {
    super(message);
    this.name = "DandoriApiError";
    this.code = code;
    this.requestId = requestId;
  }
}

export function getSafeErrorMessage(error: unknown): {
  userMessage: string;
  code: string;
  requestId: string | null;
} {
  if (error instanceof DandoriApiError) {
    return {
      userMessage: error.message || t("api_error_generic"),
      code: error.code,
      requestId: error.requestId,
    };
  }
  if (error instanceof Error) {
    return {
      userMessage: error.message || t("api_error_generic"),
      code: "client_error",
      requestId: null,
    };
  }
  return {
    userMessage: t("api_error_generic"),
    code: "unknown_error",
    requestId: null,
  };
}

async function parseApiError(response: Response): Promise<DandoriApiError> {
  try {
    const body: unknown = await response.json();
    const parsed = ApiErrorSchema.safeParse(body);
    if (parsed.success) {
      return new DandoriApiError(
        parsed.data.error.code,
        parsed.data.error.message,
        parsed.data.error.request_id
      );
    }
  } catch {
    // Không thể parse json
  }
  return nonJsonError(response.status);
}

/** Lỗi không có thân JSON của máy chủ: 5xx thường là máy chủ chưa chạy (proxy báo lỗi), còn lại là lỗi chung. */
function nonJsonError(status: number): DandoriApiError {
  const unreachable = status === 500 || status === 502 || status === 503 || status === 504;
  return new DandoriApiError(
    `http_${status}`,
    unreachable ? t("api_error_unreachable") : t("api_error_generic"),
    ""
  );
}

export async function apiClient<T>(
  endpoint: string,
  options: RequestInit = {}
): Promise<T> {
  const url = endpoint.startsWith("/") ? endpoint : `/${endpoint}`;
  const headers = new Headers(options.headers);
  if (!headers.has("Content-Type") && typeof options.body === "string") {
    headers.set("Content-Type", "application/json");
  }

  let response: Response;
  try {
    response = await fetch(url, {
      ...options,
      headers,
      credentials: "same-origin",
    });
  } catch {
    // Mất kết nối hoặc máy chủ chưa chạy: không để lộ thông báo thô của trình duyệt.
    throw new DandoriApiError("network_error", t("api_error_unreachable"), "");
  }

  const contentType = response.headers.get("content-type") || "";
  const isJson = contentType.includes("application/json");

  if (!response.ok) {
    if (response.status === 401 && !url.includes("/api/v1/auth/login") && !url.includes("/api/v1/me")) {
      if (typeof window !== "undefined") {
        window.dispatchEvent(new CustomEvent("dandori:unauthorized"));
      }
    }
    const apiError = isJson
      ? await parseApiError(response)
      : nonJsonError(response.status);
    throw apiError;
  }

  return (isJson ? await response.json() : await response.text()) as T;
}
