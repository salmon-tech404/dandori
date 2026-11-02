export * from "./types_generated";

export type MemoryCategory = "fact" | "decision" | "convention" | "lesson";
export type MemoryStatus = "pending" | "approved" | "rejected";

export interface MemoryFilters {
  status?: string;
  category?: string;
  projectId?: string;
}
