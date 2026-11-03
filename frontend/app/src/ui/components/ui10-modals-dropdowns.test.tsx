import { describe, it, expect, vi } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import { ConfirmDialog } from "./ConfirmDialog";
import { Modal } from "./Modal";
import { useDismiss } from "./useDismiss";
import { useRef, useState } from "react";

function TestDropdownComponent() {
  const [open, setOpen] = useState(true);
  const ref = useRef<HTMLDivElement>(null);
  useDismiss({ isOpen: open, onDismiss: () => setOpen(false), ref });

  return (
    <div>
      <div data-testid="outside-element">Bên ngoài</div>
      {open && (
        <div ref={ref} data-testid="dropdown-content">
          <button type="button">Mục 1</button>
        </div>
      )}
    </div>
  );
}

describe("UI-10: Modal dismiss behavior", () => {
  it("renders fixed overlay with role dialog and closes on Escape", () => {
    const onClose = vi.fn();
    render(
      <Modal open={true} onClose={onClose} title="Hộp thoại kiểm thử">
        <p>Nội dung</p>
      </Modal>
    );
    expect(screen.getByRole("dialog")).toBeDefined();
    fireEvent.keyDown(window, { key: "Escape" });
    expect(onClose).toHaveBeenCalled();
  });

  it("ConfirmDialog delegates to Modal and closes on backdrop click", () => {
    const onCancel = vi.fn();
    render(
      <ConfirmDialog
        open={true}
        title="Xác nhận"
        message="Xóa?"
        onConfirm={() => {}}
        onCancel={onCancel}
      />
    );
    fireEvent.click(screen.getByTestId("modal-backdrop"));
    expect(onCancel).toHaveBeenCalled();
  });
});

describe("UI-10: Modal conditional dismiss", () => {
  it("does not close on backdrop click when closeOnBackdrop is false", () => {
    const onClose = vi.fn();
    render(
      <Modal open={true} onClose={onClose} closeOnBackdrop={false} title="Hộp thoại">
        <p>Không đóng nền</p>
      </Modal>
    );
    fireEvent.click(screen.getByTestId("modal-backdrop"));
    expect(onClose).not.toHaveBeenCalled();
  });

  it("does not close on Escape when closeOnEscape is false", () => {
    const onClose = vi.fn();
    render(
      <Modal open={true} onClose={onClose} closeOnEscape={false} title="Hộp thoại">
        <p>Không đóng Escape</p>
      </Modal>
    );
    fireEvent.keyDown(window, { key: "Escape" });
    expect(onClose).not.toHaveBeenCalled();
  });
});

describe("UI-10: Dropdown & useDismiss behavior", () => {
  it("dismisses dropdown when clicking outside", () => {
    render(<TestDropdownComponent />);
    expect(screen.getByTestId("dropdown-content")).toBeDefined();
    fireEvent.mouseDown(screen.getByTestId("outside-element"));
    expect(screen.queryByTestId("dropdown-content")).toBeNull();
  });

  it("dismisses dropdown when pressing Escape", () => {
    render(<TestDropdownComponent />);
    expect(screen.getByTestId("dropdown-content")).toBeDefined();
    fireEvent.keyDown(window, { key: "Escape" });
    expect(screen.queryByTestId("dropdown-content")).toBeNull();
  });
});
