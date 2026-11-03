import { useEffect, RefObject } from "react";

export interface UseDismissOptions {
  onDismiss: () => void;
  isOpen: boolean;
  ref: RefObject<HTMLElement | null>;
  triggerRef?: RefObject<HTMLElement | null>;
  escape?: boolean;
  clickOutside?: boolean;
}

function createPointerHandler(
  ref: RefObject<HTMLElement | null>,
  triggerRef: RefObject<HTMLElement | null> | undefined,
  clickOutside: boolean,
  onDismiss: () => void
) {
  return (event: MouseEvent | TouchEvent) => {
    const target = event.target as Node | null;
    if (!target) return;
    if (ref.current && ref.current.contains(target)) return;
    if (triggerRef?.current && triggerRef.current.contains(target)) return;
    if (clickOutside) onDismiss();
  };
}

export function useDismiss({
  onDismiss,
  isOpen,
  ref,
  triggerRef,
  escape = true,
  clickOutside = true,
}: UseDismissOptions) {
  useEffect(() => {
    if (!isOpen) return;

    const handlePointer = createPointerHandler(ref, triggerRef, clickOutside, onDismiss);
    const handleKey = (e: KeyboardEvent) => {
      if (escape && e.key === "Escape") {
        e.stopPropagation();
        onDismiss();
      }
    };

    document.addEventListener("mousedown", handlePointer);
    document.addEventListener("touchstart", handlePointer);
    document.addEventListener("click", handlePointer);
    window.addEventListener("keydown", handleKey);

    return () => {
      document.removeEventListener("mousedown", handlePointer);
      document.removeEventListener("touchstart", handlePointer);
      document.removeEventListener("click", handlePointer);
      window.removeEventListener("keydown", handleKey);
    };
  }, [isOpen, onDismiss, ref, triggerRef, escape, clickOutside]);
}
