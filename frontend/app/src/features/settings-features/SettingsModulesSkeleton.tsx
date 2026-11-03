import { FC } from "react";
import { Skeleton } from "../../ui/components/Skeleton";

const DUMMY_GROUPS = [4, 5, 4, 6, 4, 4, 4, 3]; // 34 rows total across 8 standard groups

function SkeletonRow({ idx }: { idx: number }) {
  return (
    <div
      key={idx}
      style={{
        display: "flex",
        alignItems: "center",
        justifyContent: "space-between",
        padding: "var(--sp-12) var(--sp-16)",
        borderBottom: "1px solid var(--ln)",
        gap: "var(--sp-16)",
        height: "58px",
      }}
    >
      <div style={{ display: "flex", alignItems: "center", gap: "var(--sp-12)", flex: 1 }}>
        <Skeleton width="40px" height="22px" radius="pill" />
        <div style={{ display: "flex", flexDirection: "column", gap: "6px", flex: 1 }}>
          <Skeleton width="180px" height="16px" radius="sm" />
          <Skeleton width="320px" height="14px" radius="sm" />
        </div>
      </div>
      <Skeleton width="80px" height="24px" radius="md" />
    </div>
  );
}

function SkeletonGroup({ rowCount, idx }: { rowCount: number; idx: number }) {
  return (
    <div
      key={idx}
      style={{
        backgroundColor: "var(--sf)",
        borderRadius: "var(--radius-10)",
        border: "1px solid var(--ln)",
        marginBottom: "var(--sp-16)",
        overflow: "hidden",
      }}
    >
      <div
        style={{
          padding: "var(--sp-12) var(--sp-16)",
          backgroundColor: "var(--sf2)",
          display: "flex",
          justifyContent: "space-between",
          alignItems: "center",
        }}
      >
        <Skeleton width="160px" height="20px" radius="md" />
        <Skeleton width="60px" height="20px" radius="pill" />
      </div>
      <div>
        {Array.from({ length: rowCount }).map((_, rIdx) => (
          <SkeletonRow key={rIdx} idx={rIdx} />
        ))}
      </div>
    </div>
  );
}

export const SettingsModulesSkeleton: FC = () => {
  return (
    <div data-testid="settings-modules-skeleton">
      {DUMMY_GROUPS.map((rowCount, idx) => (
        <SkeletonGroup key={idx} rowCount={rowCount} idx={idx} />
      ))}
    </div>
  );
};
