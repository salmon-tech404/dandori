import { describe, it, expect } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import { MemoryRouter } from "react-router-dom";
import { UserMenu } from "./UserMenu";

const user = { id: "u1", username: "dandori@gmail.com", system_role: "admin" };

function setup() {
  const counts = { logout: 0, theme: 0 };
  const onLogout = () => {
    counts.logout += 1;
  };
  const onToggleTheme = () => {
    counts.theme += 1;
  };
  render(
    <MemoryRouter>
      <UserMenu user={user} theme="light" onToggleTheme={onToggleTheme} onLogout={onLogout} />
    </MemoryRouter>,
  );
  return counts;
}

describe("UserMenu (tên + bánh răng, kiểu veltrix-ai)", () => {
  it("hiện tên và vai trò, menu đóng sẵn", () => {
    setup();
    expect(screen.getByText("dandori@gmail.com")).toBeDefined();
    expect(screen.getByText("Quản trị viên")).toBeDefined();
    expect(screen.queryByRole("menu")).toBeNull();
  });

  it("bấm bánh răng mở menu; Đăng xuất gọi onLogout và đóng menu", () => {
    const counts = setup();
    fireEvent.click(screen.getByRole("button", { name: "Tài khoản và cài đặt" }));
    expect(screen.getByRole("menu")).toBeDefined();
    fireEvent.click(screen.getByRole("menuitem", { name: "Đăng xuất" }));
    expect(counts.logout).toBe(1);
    expect(screen.queryByRole("menu")).toBeNull();
  });

  it("đổi giao diện sáng/tối từ menu", () => {
    const counts = setup();
    fireEvent.click(screen.getByRole("button", { name: "Tài khoản và cài đặt" }));
    fireEvent.click(screen.getByRole("menuitem", { name: "Giao diện tối" }));
    expect(counts.theme).toBe(1);
  });

  it("nhấn Escape hoặc bấm ra ngoài thì đóng menu", () => {
    setup();
    fireEvent.click(screen.getByRole("button", { name: "Tài khoản và cài đặt" }));
    fireEvent.keyDown(document, { key: "Escape" });
    expect(screen.queryByRole("menu")).toBeNull();
    fireEvent.click(screen.getByRole("button", { name: "Tài khoản và cài đặt" }));
    fireEvent.mouseDown(document.body);
    expect(screen.queryByRole("menu")).toBeNull();
  });
});
