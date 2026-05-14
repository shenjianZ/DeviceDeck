import { create } from "zustand";
import type { Page } from "../types";

interface PageStore {
  page: Page;
  setPage: (p: Page) => void;
}

export const usePageStore = create<PageStore>((set) => ({
  page: "dashboard",
  setPage: (page) => set({ page }),
}));
