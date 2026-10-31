import { ReactNode } from "react";
import { t } from "../../core/i18n";

export interface Column<T> {
  key: string;
  header: string;
  render?: (item: T) => ReactNode;
  width?: string;
}

export interface TableProps<T> {
  columns: Column<T>[];
  data: T[];
  loading?: boolean;
  error?: string | null;
  emptyMessage?: string;
  onRowClick?: (item: T) => void;
}

function TableHead<T>({ columns }: { columns: Column<T>[] }) {
  return (
    <thead>
      <tr style={{ backgroundColor: "var(--sf2)", borderBottom: "1px solid var(--ln)" }}>
        {columns.map((col) => (
          <th key={col.key} style={{ padding: "10px 14px", fontWeight: 600, fontSize: "var(--fs-13)", color: "var(--mu)", width: col.width }}>
            {col.header}
          </th>
        ))}
      </tr>
    </thead>
  );
}

function TableBody<T extends { id?: string | number }>({ columns, data, onRowClick }: { columns: Column<T>[]; data: T[]; onRowClick?: (item: T) => void }) {
  return (
    <tbody>
      {data.map((item, idx) => (
        <tr
          key={item.id ?? idx}
          tabIndex={onRowClick ? 0 : undefined}
          role={onRowClick ? "button" : undefined}
          onClick={() => onRowClick?.(item)}
          onKeyDown={(e) => {
            if (onRowClick && (e.key === "Enter" || e.key === " ")) {
              e.preventDefault();
              onRowClick(item);
            }
          }}
          style={{ borderBottom: idx === data.length - 1 ? "none" : "1px solid var(--ln)", cursor: onRowClick ? "pointer" : "default" }}
        >
          {columns.map((col) => (
            <td key={col.key} style={{ padding: "12px 14px", color: "var(--fg)" }}>
              {col.render ? col.render(item) : String((item as Record<string, unknown>)[col.key] ?? "")}
            </td>
          ))}
        </tr>
      ))}
    </tbody>
  );
}

export function Table<T extends { id?: string | number }>({ columns, data, loading, error, emptyMessage, onRowClick }: TableProps<T>) {
  if (loading) {
    return <div role="status" style={{ padding: "var(--sp-24)", textAlign: "center", color: "var(--mu)" }}>{t("loading")}</div>;
  }
  if (error) {
    return <div role="alert" style={{ padding: "var(--sp-16)", backgroundColor: "var(--rdb)", color: "var(--rd)", textAlign: "center" }}>{error}</div>;
  }
  if (data.length === 0) {
    return <div style={{ padding: "var(--sp-32)", textAlign: "center", color: "var(--mu)", backgroundColor: "var(--sf)", border: "1px dashed var(--ln)" }}>{emptyMessage || t("empty_state")}</div>;
  }
  return (
    <div style={{ width: "100%", overflowX: "auto", border: "1px solid var(--ln)", borderRadius: "var(--radius-10)", backgroundColor: "var(--sf)" }}>
      <table style={{ width: "100%", borderCollapse: "collapse", fontSize: "var(--fs-14)", textAlign: "left" }}>
        <TableHead columns={columns} />
        <TableBody columns={columns} data={data} onRowClick={onRowClick} />
      </table>
    </div>
  );
}
