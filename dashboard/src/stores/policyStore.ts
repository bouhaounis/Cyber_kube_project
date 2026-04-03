import { create } from 'zustand';

export interface Policy {
  id: string;
  name: string;
  description: string;
  enabled?: boolean;
  createdAt?: string;
}

interface PolicyStore {
  policies: Policy[];
  addPolicy: (policy: Policy) => void;
  updatePolicy: (id: string, policy: Partial<Policy>) => void;
  removePolicy: (id: string) => void;
}

export const usePolicyStore = create<PolicyStore>((set) => ({
  policies: [],
  addPolicy: (policy) => set((state) => ({ policies: [...state.policies, policy] })),
  updatePolicy: (id, updates) =>
    set((state) => ({
      policies: state.policies.map((p) => (p.id === id ? { ...p, ...updates } : p)),
    })),
  removePolicy: (id) => set((state) => ({ policies: state.policies.filter((p) => p.id !== id) })),
}));
