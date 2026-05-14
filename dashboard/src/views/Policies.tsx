import { useEffect, useState } from 'react';
import { Edit, Plus, Shield, Trash2 } from 'lucide-react';
import toast from 'react-hot-toast';
import { usePolicyStore } from '../stores/policyStore';
import { DataTable } from '../components/tables/DataTable';
import { apiClient } from '../api/client';
import type { Policy as ApiPolicy } from '../api/types';
import type { Policy as StorePolicy } from '../stores/policyStore';

export default function Policies() {
  const { policies, addPolicy, removePolicy, updatePolicy } = usePolicyStore();
  const [isModalOpen, setIsModalOpen] = useState(false);
  const [editingPolicy, setEditingPolicy] = useState<StorePolicy | null>(null);

  useEffect(() => {
    loadPolicies();
  }, []);

  const loadPolicies = async () => {
    try {
      const data = await apiClient.get<ApiPolicy[]>('/api/v1/policies');
      data.forEach((p) => addPolicy(p));
    } catch (error) {
      console.error('Failed to load policies:', error);
      toast.error('Failed to load policies');
    }
  };

  const handleDelete = async (id: string) => {
    if (window.confirm('Are you sure you want to delete this policy?')) {
      try {
        await apiClient.delete(`/api/v1/policies/${id}`);
        removePolicy(id);
        toast.success('Policy deleted');
      } catch (error) {
        toast.error('Failed to delete policy');
      }
    }
  };

  const columns = [
    {
      key: 'name',
      header: 'Policy',
      sortable: true,
      render: (policy: StorePolicy) => (
        <div>
          <p className="font-semibold text-white">{policy.name}</p>
          <p className="mt-1 text-xs uppercase tracking-[0.18em] text-slate-500">{policy.id}</p>
        </div>
      ),
    },
    {
      key: 'description',
      header: 'Description',
      sortable: true,
      render: (policy: StorePolicy) => <span className="text-slate-300">{policy.description}</span>,
    },
    {
      key: 'created_at',
      header: 'Created',
      sortable: true,
      render: (policy: StorePolicy) => (
        <span className="font-mono text-xs text-slate-400">
          {policy.createdAt ? new Date(policy.createdAt).toLocaleDateString() : 'N/A'}
        </span>
      ),
    },
    {
      key: 'actions',
      header: 'Actions',
      render: (policy: StorePolicy) => (
        <div className="flex items-center gap-2">
          <button
            onClick={() => {
              setEditingPolicy(policy);
              setIsModalOpen(true);
            }}
            className="cursor-pointer rounded-2xl border border-sky-400/15 bg-sky-500/10 p-2 text-sky-200 hover:bg-sky-500/20"
          >
            <Edit className="h-4 w-4" />
          </button>
          <button
            onClick={() => handleDelete(policy.id)}
            className="cursor-pointer rounded-2xl border border-rose-400/15 bg-rose-500/10 p-2 text-rose-200 hover:bg-rose-500/20"
          >
            <Trash2 className="h-4 w-4" />
          </button>
        </div>
      ),
    },
  ];

  return (
    <div className="page-grid">
      <section className="flex flex-col gap-5 lg:flex-row lg:items-end lg:justify-between">
        <div>
          <div className="status-pill mb-4">Policy intelligence</div>
          <h1 className="section-title">Security policies</h1>
          <p className="mt-2 max-w-2xl text-sm leading-7 text-slate-400">
            Manage the ruleset that governs runtime behavior, escalation logic, and enforcement posture across the cluster.
          </p>
        </div>
        <button
          onClick={() => {
            setEditingPolicy(null);
            setIsModalOpen(true);
          }}
          className="action-button action-button-primary"
        >
          <Plus className="h-4 w-4" />
          Create policy
        </button>
      </section>

      <section className="grid gap-4 xl:grid-cols-[0.95fr_1.05fr]">
        <div className="surface-card p-6">
          <div className="flex items-center gap-4">
            <div className="rounded-[22px] border border-sky-400/18 bg-gradient-to-br from-sky-500/18 to-violet-500/14 p-4">
              <Shield className="h-6 w-6 text-sky-200" />
            </div>
            <div>
              <p className="text-sm text-slate-400">Active policies</p>
              <p className="mt-2 text-4xl font-semibold text-white">{policies.length}</p>
            </div>
          </div>
          <div className="mt-6 grid gap-3 md:grid-cols-2">
            <div className="rounded-[22px] border border-white/10 bg-slate-950/40 p-4">
              <p className="text-xs uppercase tracking-[0.18em] text-slate-500">Coverage</p>
              <p className="mt-3 text-2xl font-semibold text-white">All namespaces</p>
            </div>
            <div className="rounded-[22px] border border-white/10 bg-slate-950/40 p-4">
              <p className="text-xs uppercase tracking-[0.18em] text-slate-500">Sync status</p>
              <p className="mt-3 text-2xl font-semibold text-emerald-300">Healthy</p>
            </div>
          </div>
        </div>

        <div className="surface-card p-6">
          <p className="text-xs uppercase tracking-[0.22em] text-slate-500">Control guidance</p>
          <h2 className="mt-2 text-xl font-semibold text-white">Recommended review rhythm</h2>
          <div className="mt-5 space-y-3">
            {[
              'Review drift-prone policies after every engine deployment.',
              'Use narrow descriptions so alerts map cleanly to intended control behavior.',
              'Retire duplicate rules early to keep response queues readable.',
            ].map((item) => (
              <div key={item} className="rounded-[22px] border border-white/10 bg-slate-950/40 px-4 py-4 text-sm leading-6 text-slate-300">
                {item}
              </div>
            ))}
          </div>
        </div>
      </section>

      <section>
        <DataTable data={policies} columns={columns} searchable pagination pageSize={10} />
      </section>

      {isModalOpen && (
        <PolicyModal
          policy={editingPolicy}
          onClose={() => {
            setIsModalOpen(false);
            setEditingPolicy(null);
          }}
          onSave={async (policyData: Partial<StorePolicy>) => {
            try {
              if (editingPolicy) {
                await apiClient.put(`/api/v1/policies/${editingPolicy.id}`, policyData);
                updatePolicy(editingPolicy.id, policyData);
                toast.success('Policy updated');
              } else {
                const created = await apiClient.post<ApiPolicy>('/api/v1/policies', policyData);
                addPolicy(created);
                toast.success('Policy created');
              }
              setIsModalOpen(false);
              setEditingPolicy(null);
            } catch (error) {
              const message = error instanceof Error ? error.message : 'Failed to save policy';
              toast.error(message);
            }
          }}
        />
      )}
    </div>
  );
}

function PolicyModal({
  policy,
  onClose,
  onSave,
}: {
  policy: StorePolicy | null;
  onClose: () => void;
  onSave: (data: Partial<StorePolicy>) => void;
}) {
  const [formData, setFormData] = useState({
    id: policy?.id || '',
    name: policy?.name || '',
    description: policy?.description || '',
  });

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-slate-950/80 px-4 backdrop-blur-sm">
      <div className="surface-card-strong w-full max-w-xl p-6 md:p-7">
        <div className="mb-6">
          <p className="text-xs uppercase tracking-[0.22em] text-slate-500">Policy editor</p>
          <h2 className="mt-2 text-2xl font-semibold text-white">{policy ? 'Edit policy' : 'Create policy'}</h2>
        </div>

        <div className="space-y-4">
          <div>
            <label className="mb-2 block text-sm font-medium text-slate-300">ID</label>
            <input
              type="text"
              value={formData.id}
              onChange={(e) => setFormData({ ...formData, id: e.target.value })}
              className="app-input"
              disabled={!!policy}
            />
            <p className="mt-2 text-xs leading-6 text-slate-500">
              Optional. You can leave this blank and the API will generate a valid ID automatically.
            </p>
          </div>
          <div>
            <label className="mb-2 block text-sm font-medium text-slate-300">Name</label>
            <input
              type="text"
              value={formData.name}
              onChange={(e) => setFormData({ ...formData, name: e.target.value })}
              className="app-input"
            />
          </div>
          <div>
            <label className="mb-2 block text-sm font-medium text-slate-300">Description</label>
            <textarea
              value={formData.description}
              onChange={(e) => setFormData({ ...formData, description: e.target.value })}
              rows={4}
              className="app-input resize-none"
            />
          </div>
        </div>

        <div className="mt-6 flex justify-end gap-3">
          <button onClick={onClose} className="action-button">
            Cancel
          </button>
          <button onClick={() => onSave(formData)} className="action-button action-button-primary">
            Save policy
          </button>
        </div>
      </div>
    </div>
  );
}
