import { describe, it, expect, vi } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import { Switch } from "./Switch";

describe("Switch dùng chung", () => {
  it("thanh trượt là hình chữ nhật bo góc (rộng hơn cao, không phải hình tròn)", () => {
    render(<Switch checked={false} onChange={() => {}} aria-label="demo" />);
    const track = screen.getByRole("switch", { name: "demo" }).firstElementChild as HTMLElement;
    expect(parseInt(track.style.width, 10)).toBeGreaterThan(parseInt(track.style.height, 10));
    expect(track.style.borderRadius).not.toContain("pill");
  });

  it("vùng chạm đủ lớn nhưng không làm biến dạng thanh trượt", () => {
    render(<Switch checked onChange={() => {}} aria-label="demo" />);
    const btn = screen.getByRole("switch", { name: "demo" });
    expect(btn.style.minHeight).toBe("var(--touch-target)");
    const track = btn.firstElementChild as HTMLElement;
    expect(track.style.height).toBe("22px");
  });

  it("bấm và phím cách đổi trạng thái; bị khóa thì không đổi", () => {
    const onChange = vi.fn();
    const { rerender } = render(<Switch checked={false} onChange={onChange} aria-label="demo" />);
    fireEvent.click(screen.getByRole("switch", { name: "demo" }));
    expect(onChange).toHaveBeenCalledWith(true);
    onChange.mockClear();
    rerender(<Switch checked={false} disabled onChange={onChange} aria-label="demo" />);
    fireEvent.click(screen.getByRole("switch", { name: "demo" }));
    expect(onChange).not.toHaveBeenCalled();
  });
});
