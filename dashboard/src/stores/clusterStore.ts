import { create } from 'zustand';

export interface Pod {
  id: string;
  name: string;
  namespace: string;
  status: string;
  node: string;
}

interface ClusterStore {
  pods: Pod[];
  selectedPod: Pod | null;
  setPods: (pods: Pod[]) => void;
  selectPod: (pod: Pod | null) => void;
}

export const useClusterStore = create<ClusterStore>((set) => ({
  pods: [],
  selectedPod: null,
  setPods: (pods) => set({ pods }),
  selectPod: (pod) => set({ selectedPod: pod }),
}));
