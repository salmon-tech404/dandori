import { SharedMemoryPanel } from "./SharedMemoryPanel";

export { SharedMemoryPanel } from "./SharedMemoryPanel";
export * from "./types";
export * from "./api";
export * from "./useSharedMemory";

export const contributions = {
  "shared-memory-tab": SharedMemoryPanel,
};

export default contributions;
