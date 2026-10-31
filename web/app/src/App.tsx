import React from "react";

export const App: React.FC = () => {
  return (
    <main style={{ minHeight: "100vh", display: "flex", flexDirection: "column", alignItems: "center", justifyContent: "center", padding: "2rem" }}>
      <header style={{ textAlign: "center", marginBottom: "2rem" }}>
        <h1 style={{ fontSize: "2rem", fontWeight: 700, marginBottom: "0.5rem" }}>Dandori</h1>
        <p style={{ color: "var(--color-text-secondary)" }}>
          Trung tâm điều phối Agent Code cục bộ
        </p>
      </header>
      <section style={{ maxWidth: "600px", border: "1px solid var(--color-border)", borderRadius: "var(--radius-lg)", padding: "1.5rem", backgroundColor: "var(--color-bg-secondary)" }}>
        <h2 style={{ fontSize: "1.25rem", fontWeight: 600, marginBottom: "1rem" }}>Hệ thống sẵn sàng</h2>
        <p style={{ fontSize: "0.875rem", color: "var(--color-text-secondary)", marginBottom: "1rem" }}>
          Khung ứng dụng web đã được khởi tạo thành công. Các module chức năng sẽ được nạp động theo cấu hình tại Feature Registry.
        </p>
        <button
          type="button"
          style={{
            minHeight: "var(--touch-target-min)",
            minWidth: "var(--touch-target-min)",
            padding: "0.5rem 1rem",
            backgroundColor: "var(--color-primary)",
            color: "#ffffff",
            border: "none",
            borderRadius: "var(--radius-md)",
            cursor: "pointer",
            fontWeight: 500,
          }}
          onClick={() => {
            // Placeholder action
          }}
        >
          Bắt đầu
        </button>
      </section>
    </main>
  );
};

export default App;
