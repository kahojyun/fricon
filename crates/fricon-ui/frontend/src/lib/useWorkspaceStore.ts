import { create } from "zustand";

interface WorkspaceState {
  path: string;
  setPath: (path: string) => void;
}

export const useWorkspaceStore = create<WorkspaceState>((set) => ({
  path: "(no workspace)",
  setPath: (path) => set({ path }),
}));
