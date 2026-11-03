import { SCREEN_SLOTS_MAP } from "./useFeatures";
import { SLOT_CONTRIBUTIONS } from "./features_generated";

/**
 * Tải trước các module đóng góp cho một màn hình khi người dùng rê chuột hoặc focus vào mục menu.
 * Giúp màn hình xuất hiện tức thì mà không giật layout.
 */
export function preloadScreen(screenId: string): void {
  const slots = SCREEN_SLOTS_MAP[screenId] || [];
  for (const slot of slots) {
    const contribs = SLOT_CONTRIBUTIONS[slot] || [];
    for (const c of contribs) {
      try {
        void c.load();
      } catch {
        // Bỏ qua lỗi nạp trước
      }
    }
  }
}
