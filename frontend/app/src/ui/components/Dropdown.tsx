import { FC, ReactNode, useState, useRef, cloneElement, isValidElement, CSSProperties, KeyboardEvent, RefObject } from "react";
import { useDismiss } from "./useDismiss";

export interface DropdownProps {
  trigger: ReactNode | ((props: { isOpen: boolean; toggle: () => void }) => ReactNode);
  content: ReactNode;
  align?: "left" | "right";
  open?: boolean;
  onOpenChange?: (open: boolean) => void;
  className?: string;
  style?: CSSProperties;
}

function renderDropdownTrigger(trigger: DropdownProps["trigger"], isOpen: boolean, toggle: () => void): ReactNode {
  if (typeof trigger === "function") return trigger({ isOpen, toggle });
  if (isValidElement(trigger)) {
    const triggerEl = trigger as React.ReactElement<{ onClick?: (e: React.MouseEvent) => void; "aria-haspopup"?: string | boolean; "aria-expanded"?: boolean }>;
    const prevOnClick = triggerEl.props?.onClick;
    return cloneElement(triggerEl, {
      onClick: (e: React.MouseEvent) => {
        if (typeof prevOnClick === "function") prevOnClick(e);
        toggle();
      },
      "aria-haspopup": "true",
      "aria-expanded": isOpen,
    });
  }
  return <div onClick={toggle} style={{ cursor: "pointer" }} role="button" tabIndex={0}>{trigger}</div>;
}

function handleArrowNavigation(e: KeyboardEvent<HTMLDivElement>, menuRef: RefObject<HTMLDivElement | null>) {
  if (e.key !== "ArrowDown" && e.key !== "ArrowUp") return;
  e.preventDefault();
  const items = menuRef.current?.querySelectorAll<HTMLElement>('button:not([disabled]), [href], [tabindex]:not([tabindex="-1"])');
  if (!items || items.length === 0) return;
  const currentIndex = Array.from(items).indexOf(document.activeElement as HTMLElement);
  const nextIndex = e.key === "ArrowDown"
    ? (currentIndex < items.length - 1 ? currentIndex + 1 : 0)
    : (currentIndex > 0 ? currentIndex - 1 : items.length - 1);
  items[nextIndex]?.focus();
}

function buildMenuStyle(align: "left" | "right", isOpen: boolean): CSSProperties {
  return {
    position: "absolute",
    top: "calc(100% + var(--sp-4))",
    ...(align === "right" ? { right: 0 } : { left: 0 }),
    backgroundColor: "var(--sf)",
    border: "1px solid var(--ln)",
    borderRadius: "var(--radius-8)",
    boxShadow: "0 4px 16px rgba(0, 0, 0, 0.12)",
    zIndex: 100,
    minWidth: "160px",
    padding: "var(--sp-4)",
    display: isOpen ? "block" : "none",
  };
}

export const Dropdown: FC<DropdownProps> = ({ trigger, content, align = "left", open: controlledOpen, onOpenChange, className = "", style }) => {
  const [internalOpen, setInternalOpen] = useState(false);
  const isControlled = controlledOpen !== undefined;
  const isOpen = isControlled ? controlledOpen : internalOpen;
  const triggerRef = useRef<HTMLDivElement>(null);
  const menuRef = useRef<HTMLDivElement>(null);

  const toggle = () => {
    const next = !isOpen;
    if (!isControlled) setInternalOpen(next);
    onOpenChange?.(next);
  };

  useDismiss({ isOpen, onDismiss: () => { if (!isControlled) setInternalOpen(false); onOpenChange?.(false); }, ref: menuRef, triggerRef });

  return (
    <div ref={triggerRef} style={{ position: "relative", display: "inline-block", ...style }} className={`dandori-dropdown ${className}`} onKeyDown={(e) => { if (isOpen) handleArrowNavigation(e, menuRef); }}>
      {renderDropdownTrigger(trigger, isOpen, toggle)}
      {isOpen && <div ref={menuRef} style={buildMenuStyle(align, isOpen)} role="menu">{content}</div>}
    </div>
  );
};
