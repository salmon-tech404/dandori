import { FC, ReactNode, useEffect, useState } from "react";
import { SLOT_CONTRIBUTIONS, SlotContribution } from "./features_generated";
import { useFeatures } from "./useFeatures";
import { ErrorBoundary } from "./ErrorBoundary";

export interface SlotProps {
  name: string;
  context?: Record<string, unknown>;
  fallback?: ReactNode;
}

interface LoadedContribution {
  id: string;
  label: string;
  Component: React.ComponentType<{ context?: Record<string, unknown> }>;
}

async function loadContribution(contrib: SlotContribution): Promise<LoadedContribution | null> {
  try {
    const mod = (await contrib.load()) as {
      default?: unknown;
      contributions?: Record<string, React.ComponentType<{ context?: Record<string, unknown> }>>;
    };
    let Comp: React.ComponentType<{ context?: Record<string, unknown> }> | undefined;
    if (typeof mod.default === "function") {
      Comp = mod.default as React.ComponentType<{ context?: Record<string, unknown> }>;
    } else if (mod.contributions && typeof mod.contributions[contrib.id] === "function") {
      Comp = mod.contributions[contrib.id];
    } else if (mod.default && typeof (mod.default as Record<string, unknown>)[contrib.id] === "function") {
      Comp = (mod.default as Record<string, unknown>)[contrib.id] as React.ComponentType<{ context?: Record<string, unknown> }>;
    }
    if (Comp) return { id: contrib.id, label: contrib.label, Component: Comp };
  } catch (err) {
    console.error(`Không thể nạp ô cắm ${contrib.id}:`, err);
  }
  return null;
}

export const Slot: FC<SlotProps> = ({ name, context, fallback = null }) => {
  const { isModuleEnabled } = useFeatures();
  const [loaded, setLoaded] = useState<LoadedContribution[]>([]);
  const activeContribs = (SLOT_CONTRIBUTIONS[name] || [])
    .filter((c) => isModuleEnabled(c.moduleId))
    .sort((a, b) => a.order - b.order);

  useEffect(() => {
    let mounted = true;
    if (activeContribs.length === 0) {
      setLoaded([]);
      return;
    }
    Promise.all(activeContribs.map(loadContribution)).then((res) => {
      if (mounted) setLoaded(res.filter((r): r is LoadedContribution => r !== null));
    });
    return () => {
      mounted = false;
    };
  }, [name, JSON.stringify(activeContribs.map((c) => c.id))]);

  if (activeContribs.length === 0) return <>{fallback}</>;
  return (
    <>
      {loaded.map(({ id, label, Component }) => (
        <ErrorBoundary key={id} name={label}>
          <Component context={context} />
        </ErrorBoundary>
      ))}
    </>
  );
};
