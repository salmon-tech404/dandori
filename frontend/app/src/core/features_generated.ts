// TỆP SINH TỰ ĐỘNG bởi scripts/gen-modules.mjs (T05); không sửa tay

export interface SlotContribution {
  moduleId: string;
  id: string;
  slot: string;
  label: string;
  order: number;
  load: () => Promise<unknown>;
}

export const SLOT_CONTRIBUTIONS: Record<string, SlotContribution[]> = {
  "board.card.badge": [
    {
      moduleId: "sample-hello",
      id: "sample-badge",
      slot: "board.card.badge",
      label: "Mẫu",
      order: 99,
      load: () => import("../features/sample-hello"),
    },
  ],
};
