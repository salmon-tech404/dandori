import { describe, it, expect } from "vitest";
import { render, screen } from "@testing-library/react";
import { CostOverviewCard } from "./CostOverviewCard";
import { BudgetBar } from "./BudgetBar";
import type { TaskCostSummary } from "./types";

describe("CostOverviewCard With Data", () => {
  it("renders total tokens and calculated cost accurately", () => {
    const summary: TaskCostSummary = {
      task_id: "task-1",
      has_token_data: true,
      total_input_tokens: 3000,
      total_output_tokens: 1500,
      total_tokens: 4500,
      total_cost: 0.0315,
      missing_runs_count: 0,
      budget: 1.0,
      is_over_budget: false,
      runs: [],
    };
    render(<CostOverviewCard summary={summary} />);
    expect(screen.getByText("4,500")).toBeDefined();
    expect(screen.getByText("$0.0315")).toBeDefined();
  });
});

describe("CostOverviewCard Missing Runs", () => {
  it("renders notice for missing runs", () => {
    const summary: TaskCostSummary = {
      task_id: "task-2",
      has_token_data: true,
      total_input_tokens: 1000,
      total_output_tokens: 500,
      total_tokens: 1500,
      total_cost: null,
      missing_runs_count: 2,
      budget: null,
      is_over_budget: false,
      runs: [],
    };
    render(<CostOverviewCard summary={summary} />);
    expect(screen.getByText("Có 2 lần chạy không có số liệu token từ CLI")).toBeDefined();
    expect(screen.getByText("Chưa cấu hình bảng giá")).toBeDefined();
  });
});

describe("CostOverviewCard No Data", () => {
  it("renders không có số liệu state when no tokens recorded", () => {
    const summary: TaskCostSummary = {
      task_id: "task-3",
      has_token_data: false,
      total_input_tokens: 0,
      total_output_tokens: 0,
      total_tokens: 0,
      total_cost: null,
      missing_runs_count: 1,
      budget: null,
      is_over_budget: false,
      runs: [],
    };
    render(<CostOverviewCard summary={summary} />);
    expect(screen.getByText("Không có số liệu")).toBeDefined();
  });
});

describe("BudgetBar Over Budget", () => {
  it("renders over budget badge when is_over_budget is true", () => {
    const summary: TaskCostSummary = {
      task_id: "task-4",
      has_token_data: true,
      total_input_tokens: 10000,
      total_output_tokens: 5000,
      total_tokens: 15000,
      total_cost: 2.5,
      missing_runs_count: 0,
      budget: 2.0,
      is_over_budget: true,
      runs: [],
    };
    render(<BudgetBar summary={summary} onOpenBudgetModal={() => {}} />);
    expect(screen.getByText("Vượt ngân sách")).toBeDefined();
    expect(screen.getByText("$2.00")).toBeDefined();
  });
});
