import { describe, it, expect, vi } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import {
  Button,
  IconButton,
  Input,
  Textarea,
  Select,
  Checkbox,
  Modal,
  Dropdown,
  Tooltip,
  Tabs,
  Card,
  Chip,
  Badge,
  Spinner,
  Skeleton,
  PageHeader,
  FormField,
} from "./index";

describe("Button & IconButton", () => {
  it("supports all variants including outline and danger", () => {
    render(
      <>
        <Button variant="primary">Chính</Button>
        <Button variant="outline">Đường viền</Button>
        <Button variant="ghost">Trong suốt</Button>
        <Button variant="danger">Nguy hiểm</Button>
      </>
    );
    expect(screen.getByRole("button", { name: "Chính" })).toBeDefined();
    expect(screen.getByRole("button", { name: "Đường viền" })).toBeDefined();
    expect(screen.getByRole("button", { name: "Trong suốt" })).toBeDefined();
    expect(screen.getByRole("button", { name: "Nguy hiểm" })).toBeDefined();
  });

  it("IconButton renders icon with mandatory aria-label and handles click", () => {
    let clicked = false;
    render(
      <IconButton
        aria-label="Xóa mục"
        icon={<span data-testid="test-icon">X</span>}
        onClick={() => { clicked = true; }}
      />
    );
    const btn = screen.getByRole("button", { name: "Xóa mục" });
    expect(btn).toBeDefined();
    expect(screen.getByTestId("test-icon")).toBeDefined();
    fireEvent.click(btn);
    expect(clicked).toBe(true);
  });

  it("IconButton respects disabled and loading states", () => {
    const { rerender } = render(
      <IconButton aria-label="Đang tải" icon={<span>*</span>} loading={true} />
    );
    expect(screen.getByRole("button", { name: "Đang tải" }).getAttribute("aria-busy")).toBe("true");

    rerender(<IconButton aria-label="Bị khóa" icon={<span>*</span>} disabled={true} />);
    expect(screen.getByRole("button", { name: "Bị khóa" }).hasAttribute("disabled")).toBe(true);
  });
});

describe("Input & Textarea", () => {
  it("Input renders label and handles value changes", () => {
    let val = "";
    render(<Input label="Tên người dùng" onChange={(e) => { val = e.target.value; }} />);
    const input = screen.getByLabelText("Tên người dùng");
    fireEvent.change(input, { target: { value: "alice" } });
    expect(val).toBe("alice");
  });

  it("Textarea renders label, handles change, and displays error/hint", () => {
    let text = "";
    const { rerender } = render(
      <Textarea label="Ghi chú" hint="Tối đa 200 ký tự" onChange={(e) => { text = e.target.value; }} />
    );
    const textarea = screen.getByLabelText("Ghi chú");
    fireEvent.change(textarea, { target: { value: "Nội dung" } });
    expect(text).toBe("Nội dung");
    expect(screen.getByText("Tối đa 200 ký tự")).toBeDefined();

    rerender(<Textarea label="Ghi chú" error="Không được để trống" />);
    expect(screen.getByRole("alert").textContent).toBe("Không được để trống");
    expect(textarea.getAttribute("aria-invalid")).toBe("true");
  });
});

describe("Select & Checkbox", () => {
  it("Select renders options and triggers onChange", () => {
    let selected = "b";
    render(
      <Select
        label="Vai trò"
        value={selected}
        options={[
          { value: "a", label: "Quản trị" },
          { value: "b", label: "Thành viên" },
        ]}
        onChange={(e) => { selected = e.target.value; }}
      />
    );
    const select = screen.getByLabelText("Vai trò");
    expect(select).toBeDefined();
    fireEvent.change(select, { target: { value: "a" } });
    expect(selected).toBe("a");
  });

  it("Checkbox renders label and toggles checked state", () => {
    let checked = false;
    render(
      <Checkbox label="Đồng ý điều khoản" checked={checked} onChange={(e) => { checked = e.target.checked; }} />
    );
    const checkbox = screen.getByLabelText("Đồng ý điều khoản");
    expect(checkbox).toBeDefined();
    fireEvent.click(checkbox);
    expect(checked).toBe(true);
  });
});

describe("Modal Component", () => {
  it("renders modal dialog when open and traps focus", () => {
    const onClose = vi.fn();
    render(
      <Modal open={true} title="Tiêu đề Modal" onClose={onClose}>
        <p>Nội dung hộp thoại</p>
      </Modal>
    );
    expect(screen.getByRole("dialog")).toBeDefined();
    expect(screen.getByText("Tiêu đề Modal")).toBeDefined();
    expect(screen.getByText("Nội dung hộp thoại")).toBeDefined();
  });

  it("closes on Escape key press", () => {
    const onClose = vi.fn();
    render(<Modal open={true} title="Hộp thoại Escape" onClose={onClose}><p>Nội dung</p></Modal>);
    fireEvent.keyDown(window, { key: "Escape" });
    expect(onClose).toHaveBeenCalled();
  });

  it("closes on backdrop click if closeOnBackdrop is true", () => {
    const onClose = vi.fn();
    render(<Modal open={true} title="Hộp thoại Backdrop" onClose={onClose} closeOnBackdrop={true}><p>Nội dung</p></Modal>);
    const backdrop = screen.getByTestId("modal-backdrop");
    fireEvent.click(backdrop);
    expect(onClose).toHaveBeenCalled();
  });
});

describe("Dropdown Component & useDismiss", () => {
  it("toggles dropdown menu on trigger click and closes on outside click", () => {
    render(
      <div>
        <span data-testid="outside">Bên ngoài</span>
        <Dropdown trigger={<button>Mở menu</button>} content={<div data-testid="dropdown-content">Nội dung menu</div>} />
      </div>
    );
    expect(screen.queryByTestId("dropdown-content")).toBeNull();
    fireEvent.click(screen.getByRole("button", { name: "Mở menu" }));
    expect(screen.getByTestId("dropdown-content")).toBeDefined();

    fireEvent.mouseDown(screen.getByTestId("outside"));
    expect(screen.queryByTestId("dropdown-content")).toBeNull();
  });

  it("closes dropdown menu on Escape key press", () => {
    render(<Dropdown trigger={<button>Tùy chọn</button>} content={<div data-testid="menu-items">Mục 1</div>} />);
    fireEvent.click(screen.getByRole("button", { name: "Tùy chọn" }));
    expect(screen.getByTestId("menu-items")).toBeDefined();
    fireEvent.keyDown(window, { key: "Escape" });
    expect(screen.queryByTestId("menu-items")).toBeNull();
  });
});

describe("Tooltip Component", () => {
  it("wraps child element and shows tooltip on hover", () => {
    render(
      <Tooltip content="Mô tả thông tin">
        <button>Rê chuột vào đây</button>
      </Tooltip>
    );
    const trigger = screen.getByRole("button", { name: "Rê chuột vào đây" });
    fireEvent.mouseEnter(trigger.parentElement || trigger);
    expect(screen.getByRole("tooltip").textContent).toBe("Mô tả thông tin");
    fireEvent.mouseLeave(trigger.parentElement || trigger);
    expect(screen.queryByRole("tooltip")).toBeNull();
  });
});

describe("Tabs Component", () => {
  it("renders tabs with ARIA roles and handles tab selection", () => {
    let active = "tab1";
    const { rerender } = render(
      <Tabs
        tabs={[{ id: "tab1", label: "Tổng quan" }, { id: "tab2", label: "Cấu hình" }]}
        activeTab={active}
        onChange={(id) => { active = id; }}
      />
    );
    const tab1 = screen.getByRole("tab", { name: /Tổng quan/ });
    const tab2 = screen.getByRole("tab", { name: /Cấu hình/ });
    expect(tab1.getAttribute("aria-selected")).toBe("true");
    expect(tab2.getAttribute("aria-selected")).toBe("false");

    fireEvent.click(tab2);
    expect(active).toBe("tab2");

    rerender(
      <Tabs
        tabs={[{ id: "tab1", label: "Tổng quan" }, { id: "tab2", label: "Cấu hình" }]}
        activeTab="tab2"
        onChange={() => {}}
      />
    );
    expect(screen.getByRole("tab", { name: /Cấu hình/ }).getAttribute("aria-selected")).toBe("true");
  });
});

describe("Card, Chip, Badge", () => {
  it("Card renders title, subtitle, and responds to click", () => {
    let clicked = false;
    render(
      <Card title="Thẻ mẫu" subtitle="Mô tả thẻ" onClick={() => { clicked = true; }}>
        <span>Nội dung</span>
      </Card>
    );
    expect(screen.getByText("Thẻ mẫu")).toBeDefined();
    expect(screen.getByText("Mô tả thẻ")).toBeDefined();
    fireEvent.click(screen.getByText("Thẻ mẫu"));
    expect(clicked).toBe(true);
  });

  it("Chip and Badge render correct color variants", () => {
    render(
      <>
        <Chip variant="blue">Đang chạy</Chip>
        <Badge variant="orange">Cần bạn</Badge>
        <Chip variant="green">Đã xong</Chip>
      </>
    );
    expect(screen.getByText("Đang chạy")).toBeDefined();
    expect(screen.getByText("Cần bạn")).toBeDefined();
    expect(screen.getByText("Đã xong")).toBeDefined();
  });
});

describe("Spinner, Skeleton, PageHeader, FormField", () => {
  it("Spinner renders with role=status and label", () => {
    render(<Spinner label="Đang tải dữ liệu..." />);
    expect(screen.getByRole("status")).toBeDefined();
    expect(screen.getByText("Đang tải dữ liệu...")).toBeDefined();
  });

  it("Skeleton renders placeholder with pulse style", () => {
    render(<Skeleton width={200} height={40} data-testid="test-skeleton" />);
    const el = screen.getByTestId("test-skeleton");
    expect(el).toBeDefined();
    expect(el.style.width).toBe("200px");
    expect(el.style.height).toBe("40px");
  });

  it("PageHeader renders title, badges, and actions", () => {
    render(
      <PageHeader
        title="Bảng điều khiển"
        subtitle="Quản lý tổng quan"
        badges={<Chip variant="green">Online</Chip>}
        actions={<Button variant="primary">Thêm</Button>}
      />
    );
    expect(screen.getByText("Bảng điều khiển")).toBeDefined();
    expect(screen.getByText("Quản lý tổng quan")).toBeDefined();
    expect(screen.getByText("Online")).toBeDefined();
    expect(screen.getByRole("button", { name: "Thêm" })).toBeDefined();
  });

  it("FormField displays label, error, and hint correctly", () => {
    render(
      <FormField label="Họ và tên" error="Bắt buộc" hint="Nhập tên thật">
        <input id="name" />
      </FormField>
    );
    expect(screen.getByText("Họ và tên")).toBeDefined();
    expect(screen.getByRole("alert").textContent).toBe("Bắt buộc");
    expect(screen.getByText("Nhập tên thật")).toBeDefined();
  });
});
