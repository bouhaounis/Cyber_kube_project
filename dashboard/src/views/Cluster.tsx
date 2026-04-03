import { Suspense, lazy, useEffect } from 'react';
import { Activity, Network, Server, ShieldCheck } from 'lucide-react';
import { useClusterStore } from '../stores/clusterStore';
import { mockPods } from '../utils/mockData';

const Cluster3D = lazy(() =>
  import('../components/threejs/Cluster3D').then((module) => ({ default: module.Cluster3D }))
);

export default function Cluster() {
  const { pods, selectedPod, setPods } = useClusterStore();

  useEffect(() => {
    if (pods.length === 0) {
      setPods(mockPods);
    }
  }, [pods.length, setPods]);

  const stats = [
    { name: 'Total pods', value: pods.length || 12, icon: Server, tone: 'text-sky-300' },
    { name: 'Active nodes', value: 3, icon: Network, tone: 'text-violet-300' },
    { name: 'Network traffic', value: '2.4 GB/s', icon: Activity, tone: 'text-cyan-300' },
    { name: 'Protected surfaces', value: '99%', icon: ShieldCheck, tone: 'text-emerald-300' },
  ];

  return (
    <div className="page-grid">
      <section>
        <div className="status-pill mb-4">Runtime topology</div>
        <h1 className="section-title">Cluster visualization</h1>
        <p className="mt-2 max-w-2xl text-sm leading-7 text-slate-400">
          Explore Kubernetes runtime structure, observe active workloads, and inspect the pod currently in focus.
        </p>
      </section>

      <section className="grid gap-4 md:grid-cols-2 xl:grid-cols-4">
        {stats.map((stat) => {
          const Icon = stat.icon;
          return (
            <div key={stat.name} className="metric-card">
              <div className="relative flex items-start justify-between">
                <div>
                  <p className="text-sm text-slate-400">{stat.name}</p>
                  <p className="mt-3 text-4xl font-semibold text-white">{stat.value}</p>
                </div>
                <div className="rounded-[22px] border border-white/10 bg-white/5 p-3">
                  <Icon className={`h-5 w-5 ${stat.tone}`} />
                </div>
              </div>
            </div>
          );
        })}
      </section>

      <section className="grid gap-6 xl:grid-cols-[1.3fr_0.7fr]">
        <div className="surface-card p-6">
          <div className="mb-4">
            <h2 className="text-xl font-semibold text-white">3D cluster topology</h2>
            <p className="mt-1 text-sm text-slate-400">Interactive visualization of your Kubernetes cluster fabric.</p>
          </div>
          <div className="h-[620px] overflow-hidden rounded-[28px] border border-white/10 bg-slate-950/50">
            <Suspense
              fallback={
                <div className="flex h-full items-center justify-center">
                  <div className="text-center">
                    <div className="pulse-dot mx-auto mb-4 h-3 w-3 rounded-full bg-sky-400" />
                    <p className="text-sm text-slate-400">Loading cluster renderer...</p>
                  </div>
                </div>
              }
            >
              <Cluster3D />
            </Suspense>
          </div>
        </div>

        <div className="space-y-6">
          <div className="surface-card p-6">
            <p className="text-xs uppercase tracking-[0.22em] text-slate-500">Cluster status</p>
            <h2 className="mt-2 text-xl font-semibold text-white">Monitoring summary</h2>
            <div className="mt-5 space-y-3">
              {[
                ['Workloads visualized', `${pods.length || 12}`],
                ['Runtime edges', '48'],
                ['Healthy agents', '8/8'],
              ].map(([label, value]) => (
                <div key={label} className="flex items-center justify-between rounded-[22px] border border-white/10 bg-slate-950/40 px-4 py-4">
                  <span className="text-sm text-slate-400">{label}</span>
                  <span className="text-lg font-semibold text-white">{value}</span>
                </div>
              ))}
            </div>
          </div>

          {selectedPod && (
            <div className="surface-card-strong p-6">
              <p className="text-xs uppercase tracking-[0.22em] text-slate-500">Focused workload</p>
              <h3 className="mt-2 text-xl font-semibold text-white">{selectedPod.name}</h3>
              <div className="mt-5 grid gap-3">
                {[
                  ['Namespace', selectedPod.namespace],
                  ['Status', selectedPod.status],
                  ['Node', selectedPod.node],
                ].map(([label, value]) => (
                  <div key={label} className="rounded-[22px] border border-white/10 bg-slate-950/45 px-4 py-4">
                    <p className="text-xs uppercase tracking-[0.18em] text-slate-500">{label}</p>
                    <p className="mt-2 font-semibold text-white">{value}</p>
                  </div>
                ))}
              </div>
            </div>
          )}
        </div>
      </section>
    </div>
  );
}
