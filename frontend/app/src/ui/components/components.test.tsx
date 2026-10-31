import { describe, it, expect } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import { Button } from "./Button";
import { Input } from "./Input";
import { ConfirmDialog } from "./ConfirmDialog";
import { Tooltip } from "./Tooltip";
import { StatusBadge } from "./StatusBadge";
import { Table } from "./Table";

describe("Button component", () => {
  it("renders children and responds to click event", () => {
    let clicked = false;
    render(<Button onClick={() => { clicked = true; }}>Bấm vào</Button>);
    const btn = screen.getByRole("button", { name: "Bấm vào" });
    fireEvent.click(btn);
    expect(clicked).toBe(true);
  });

  it("displays loading state and disables element", () => {
    render(<Button loading>Lưu</Button>);
    const btn = screen.getByRole("button");
    expect(btn.hasAttribute("disabled")).toBe(true);
    expect(btn.getAttribute("aria-busy")).toBe("true");
  });
});

describe("Input component", () => {
  it("renders label and updates value", () => {
    let val = "";
    render(<Input label="Tên người dùng" onChange={(e) => { val = e.target.value; }} />);
    const input = screen.getByLabelText("Tên người dùng");
    fireEvent.change(input, { target: { value: "admin" } });
    expect(val).toBe("admin");
  });

  it("displays error with alert role and aria-invalid", () => {
    render(<Input label="Email" error="Email không hợp lệ" />);
    const input = screen.getByLabelText("Email");
    expect(input.getAttribute("aria-invalid")).toBe("true");
    expect(screen.getByRole("alert").textContent).toBe("Email không hợp lệ");
  });
});

describe("ConfirmDialog component", () => {
  it("handles confirm and cancel actions via state", () => {
    let confirmed = false;
    let cancelled = false;
    const { rerender } = render(
      <ConfirmDialog open={true} title="Xác nhận" message="Nội dung" onConfirm={() => { confirmed = true; }} onCancel={() => { cancelled = true; }} />
    );
    fireEvent.click(screen.getByRole("button", { name: "Xác nhận" }));
    expect(confirmed).toBe(true);

    rerender(<ConfirmDialog open={true} title="Xác nhận" message="Nội dung" onConfirm={() => {}} onCancel={() => { cancelled = true; }} />);
    fireEvent.click(screen.getByRole("button", { name: "Hủy" }));
    expect(cancelled).toBe(true);
  });

  it("triggers cancel callback when Escape is pressed", () => {
    let cancelled = false;
    render(<ConfirmDialog open={true} title="Thử nghiệm" message="Nội dung" onConfirm={() => {}} onCancel={() => { cancelled = true; }} />);
    fireEvent.keyDown(window, { key: "Escape" });
    expect(cancelled).toBe(true);
  });
});

describe("Tooltip and StatusBadge components", () => {
  it("shows tooltip text on mouseEnter and focus", () => {
    render(<Tooltip content="Giải thích chi tiết" />);
    const btn = screen.getByRole("button");
    fireEvent.mouseEnter(btn);
    expect(screen.getByRole("tooltip").textContent).toBe("Giải thích chi tiết");
    fireEvent.mouseLeave(btn);
    expect(screen.queryByRole("tooltip")).toBeNull();
    fireEvent.focus(btn);
    expect(screen.getByRole("tooltip").textContent).toBe("Giải thích chi tiết");
  });

  it("maps status codes to Vietnamese localized badge text", () => {
    const { rerender } = render(<StatusBadge status="running" />);
    expect(screen.getByText("Đang chạy")).toBeDefined();
    rerender(<StatusBadge status="needs_human" />);
    expect(screen.getByText("Cần bạn")).toBeDefined();
    rerender(<StatusBadge status="done" />);
    expect(screen.getByText("Xong")).toBeDefined();
  });
});

describe("Table component", () => {
  const cols = [{ key: "id", header: "ID" }, { key: "name", header: "Tên" }];
  const data = [{ id: "1", name: "Dự án A" }, { id: "2", name: "Dự án B" }];

  it("renders data rows correctly", () => {
    render(<Table columns={cols} data={data} />);
    expect(screen.getByText("Dự án A")).toBeDefined();
    expect(screen.getByText("Dự án B")).toBeDefined();
  });

  it("renders loading, error, and empty states appropriately", () => {
    const { rerender } = render(<Table columns={cols} data={[]} loading={true} />);
    expect(screen.getByRole("status")).toBeDefined();
    rerender(<Table columns={cols} data={[]} error="Lỗi tải dữ liệu" />);
    expect(screen.getByRole("alert").textContent).toBe("Lỗi tải dữ liệu");
    rerender(<Table columns={cols} data={[]} emptyMessage="Không có dữ liệu" />);
    expect(screen.getByText("Không có dữ liệu")).toBeDefined();
  });

  it("triggers row selection when Enter key is pressed", () => {
    let selected: unknown = null;
    render(<Table columns={cols} data={data} onRowClick={(item) => { selected = item; }} />);
    const rows = screen.getAllByRole("button");
    fireEvent.keyDown(rows[0], { key: "Enter" });
    expect(selected).toEqual(data[0]);
  });
});
