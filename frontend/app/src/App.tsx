import React from "react";

export const App: React.FC = () => {
  return (
    <main
      style={{
        minHeight: "100vh",
        display: "flex",
        flexDirection: "column",
        alignItems: "center",
        justifyContent: "center",
        padding: "var(--spacing-8)",
      }}
    >
      <header
        style={{
          textAlign: "center",
          marginBottom: "var(--spacing-8)",
        }}
      >
        <h1
          style={{
            fontSize: "var(--font-size-2xl)",
            fontWeight: 700,
            marginBottom: "var(--spacing-2)",
          }}
        >
          Dandori
        </h1>
        <p style={{ color: "var(--color-text-secondary)" }}>
          Trung tâm điều phối Agent Code cục bộ
        </p>
      </header>
      <section
        style={{
          maxWidth: "var(--max-width-card)",
          border: "1px solid var(--color-border)",
          borderRadius: "var(--radius-lg)",
          padding: "var(--spacing-6)",
          backgroundColor: "var(--color-bg-secondary)",
        }}
      >
        <h2
          style={{
            fontSize: "var(--font-size-xl)",
            fontWeight: 600,
            marginBottom: "var(--spacing-4)",
          }}
        >
          Hệ thống sẵn sàng
        </h2>
        <p
          style={{
            fontSize: "var(--font-size-sm)",
            color: "var(--color-text-secondary)",
            marginBottom: "var(--spacing-4)",
          }}
        >
          Khung ứng dụng web đã được khởi tạo thành công. Các module chức năng sẽ được nạp động theo cấu hình tại Feature Registry.
        </p>
        <button
          type="button"
          style={{
            minHeight: "var(--touch-target-min)",
            minWidth: "var(--touch-target-min)",
            padding: "var(--spacing-2) var(--spacing-4)",
            backgroundColor: "var(--color-primary)",
            color: "var(--color-on-primary)",
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
