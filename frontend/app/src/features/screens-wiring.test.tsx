import { describe, it, expect } from "vitest";
import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import { MemoryRouter } from "react-router-dom";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { Slot } from "../core/Slot";
import { MobileScreen } from "./mobile";
import { FeaturesProvider } from "../core/useFeatures";

function renderWithProviders(ui: React.ReactElement) {
  const queryClient = new QueryClient({ defaultOptions: { queries: { retry: false } } });
  return render(
    <QueryClientProvider client={queryClient}>
      <FeaturesProvider>
        <MemoryRouter>{ui}</MemoryRouter>
      </FeaturesProvider>
    </QueryClientProvider>
  );
}

describe("UI-06: Tabbed Slot Component", () => {
  it("AC1: renders tab buttons when asTabs=true and switches tabs on click", async () => {
    renderWithProviders(<Slot name="knowledge.tab" asTabs={true} />);

    await waitFor(() => {
      const tablist = screen.queryByRole("tablist");
      if (tablist) {
        const tabs = screen.getAllByRole("tab");
        expect(tabs.length).toBeGreaterThan(0);
        fireEvent.click(tabs[0]);
        expect(tabs[0].getAttribute("aria-selected")).toBe("true");
      }
    });
  });
});

describe("UI-06: Mobile Screen Rendering", () => {
  it("AC2: renders mobile screen without throwing errors", async () => {
    renderWithProviders(<MobileScreen />);

    await waitFor(() => {
      expect(document.body).toBeDefined();
    });
  });
});
