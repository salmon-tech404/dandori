import { describe, it, expect } from "vitest";
import { render, screen } from "@testing-library/react";
import { App } from "./App";

describe("App", () => {
  it("renders system readiness message", () => {
    render(<App />);
    expect(screen.getByText("Dandori")).toBeDefined();
    expect(screen.getByText("Hệ thống sẵn sàng")).toBeDefined();
  });
});
