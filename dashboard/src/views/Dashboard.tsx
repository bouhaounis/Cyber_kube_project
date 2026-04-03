import { useEffect } from 'react';
import { Link } from 'react-router-dom';
import { useAlertStore } from '../stores/alertStore';
import { useClusterStore } from '../stores/clusterStore';
import { MetricsChart } from '../components/charts/MetricsChart';
import {
  Activity,
  AlertTriangle,
  ArrowUpRight,
  Network,
  Shield,
  TrendingUp,
  Waves,
} from 'lucide-react';
import { apiClient } from '../api/client';
import { WebSocketClient } from '../api/websocket';
import toast from 'react-hot-toast';
import { mockPods } from '../utils/mockData';

const metricsData = Array.from({ length: 24 }, (_, i) => ({
  name: `${i}:00`,
  value: Math.floor(Math.random() * 100),
  alerts: Math.floor(Math.random() * 20),
}));

export default function Dashboard() {
  const { alerts, addAlert } = useAlertStore();
  const { pods, setPods } = useClusterStore();

  useEffect(() => {
    if (pods.length === 0) {
      setPods(mockPods);
    }
  }, [pods.length, setPods]);

  useEffect(() => {
    const ws = new WebSocketClient();
    ws.connect((data: any) => {
      if (data.kind) {
        addAlert(data);
        toast.error(`New Alert: ${data.kind}`, {
          duration: 5000,
        });
      }
    });

    const loadData = async () => {
      try {
        await apiClient.get<any[]>('/api/v1/policies');
        const alertsData = await apiClient.get<any[]>('/api/v1/alerts');
        alertsData.forEach((alert) => addAlert(alert));
      } catch (error) {
        console.error('Failed to load data:', error);
      }
    };

    loadData();
    return () => ws.disconnect();
  }, [addAlert]);

  const highSeverityAlerts = alerts.filter((a) => a.severity === 'high').length;
  const activePolicies = 12;
  const activePods = pods.length || 8;

  const stats = [
    {
      name: 'Threat signals',
      value: alerts.length,
      change: '+12%',
      description: 'Detections entering triage',
      icon: AlertTriangle,
      accent: 'from-rose-500/20 to-orange-500/10',
      tone: 'text-rose-300',
    },
    {
      name: 'Critical severity',
      value: highSeverityAlerts,
      change: '+5%',
      description: 'High-confidence policy violations',
      icon: Shield,
      accent: 'from-amber-500/20 to-rose-500/10',
      tone: 'text-amber-300',
    },
    {
      name: 'Active policies',
      value: activePolicies,
      change: '+2%',
      description: 'Runtime enforcement rules online',
      icon: Network,
      accent: 'from-sky-500/20 to-cyan-500/10',
      tone: 'text-sky-300',
    },
    {
      name: 'Cluster pods',
      value: activePods,
      change: '+8%',
      description: 'Observed Kubernetes workloads',
      icon: Activity,
      accent: 'from-emerald-500/20 to-cyan-500/10',
      tone: 'text-emerald-300',
    },
  ];

  return (
    <div className="page-grid section-fade-in">
      <section className="grid gap-6 xl:grid-cols-[1.55fr_0.9fr] xl:gap-7">
        <div className="surface-card-strong hover-lift relative overflow-hidden p-7 md:p-9 xl:p-10">
          <div className="absolute inset-x-0 top-0 h-px bg-gradient-to-r from-transparent via-sky-400/70 to-transparent" />
          <div className="absolute -right-20 top-0 h-56 w-56 rounded-full bg-sky-500/10 blur-3xl" />
          <div className="absolute bottom-0 left-1/3 h-48 w-48 rounded-full bg-violet-500/10 blur-3xl" />

          <div className="relative section-stack">
            <div className="flex flex-wrap items-center gap-3">
              <span className="status-pill">
                <span className="pulse-dot h-2 w-2 rounded-full bg-emerald-400 shadow-[0_0_18px_rgba(74,222,128,0.75)]" />
                Sensors operational
              </span>
              <span className="data-badge">24h live telemetry</span>
            </div>

            <div className="max-w-3xl">
              <h1 className="section-title text-[2.4rem] md:text-[3.35rem] md:leading-[1.02]">
                Enterprise-grade detection visibility for every workload.
              </h1>
              <p className="mt-5 max-w-2xl text-base leading-8 text-slate-300 md:text-lg">
                Observe runtime threats, policy drift, and cluster activity from one control plane designed for fast triage and confident response.
              </p>
            </div>

            <div className="flex flex-wrap gap-3 pt-1">
              <Link to="/alerts" className="action-button action-button-primary group">
                Review live alerts
                <ArrowUpRight className="h-4 w-4 transition-transform duration-200 group-hover:-translate-y-0.5 group-hover:translate-x-0.5" />
              </Link>
              <Link to="/policies" className="action-button">
                Inspect policies
              </Link>
            </div>

            <div className="grid gap-4 md:grid-cols-3">
              <div className="list-card p-5">
                <p className="text-xs uppercase tracking-[0.22em] text-slate-500">Threat confidence</p>
                <p className="mt-3 text-[2rem] font-semibold text-white">96.2%</p>
                <p className="mt-2 text-sm text-emerald-300">Improved signal fidelity this cycle</p>
              </div>
              <div className="list-card p-5">
                <p className="text-xs uppercase tracking-[0.22em] text-slate-500">Policy propagation</p>
                <p className="mt-3 text-[2rem] font-semibold text-white">1.4s</p>
                <p className="mt-2 text-sm text-sky-300">Cluster-wide rollout median</p>
              </div>
              <div className="list-card p-5">
                <p className="text-xs uppercase tracking-[0.22em] text-slate-500">Coverage</p>
                <p className="mt-3 text-[2rem] font-semibold text-white">8 nodes</p>
                <p className="mt-2 text-sm text-violet-300">Runtime telemetry synchronized</p>
              </div>
            </div>
          </div>
        </div>

        <div className="surface-card hover-lift p-7">
          <div className="mb-7 flex items-center justify-between">
            <div>
              <p className="text-xs uppercase tracking-[0.22em] text-slate-500">Detection posture</p>
              <h2 className="mt-2 text-[1.45rem] font-semibold text-white">Operational priorities</h2>
            </div>
            <Waves className="h-5 w-5 text-sky-300" />
          </div>

          <div className="space-y-4">
            {[
              { label: 'Critical alert response', value: '4 min', tone: 'text-rose-300' },
              { label: 'Policy drift anomalies', value: '03', tone: 'text-amber-300' },
              { label: 'Protected namespaces', value: '12', tone: 'text-sky-300' },
              { label: 'Healthy agent heartbeat', value: '99.98%', tone: 'text-emerald-300' },
            ].map((item) => (
              <div key={item.label} className="list-card flex items-center justify-between px-5 py-4">
                <span className="text-sm leading-6 text-slate-400">{item.label}</span>
                <span className={`text-xl font-semibold ${item.tone}`}>{item.value}</span>
              </div>
            ))}
          </div>
        </div>
      </section>

      <section className="grid gap-4 md:grid-cols-2 xl:grid-cols-4 xl:gap-5">
        {stats.map((stat) => {
          const Icon = stat.icon;
          return (
            <div key={stat.name} className="metric-card">
              <div className="relative flex h-full items-start justify-between gap-4">
                <div className="flex h-full max-w-[15rem] flex-col">
                  <p className="text-sm font-medium text-slate-400">{stat.name}</p>
                  <p className="mt-3 text-[2.45rem] font-semibold leading-none text-white">{stat.value}</p>
                  <div className="mt-4 flex items-center gap-2 text-sm text-emerald-300">
                    <TrendingUp className="h-4 w-4" />
                    <span>{stat.change}</span>
                  </div>
                  <p className="mt-auto pt-5 text-sm leading-6 text-slate-500">{stat.description}</p>
                </div>
                <div className={`icon-chip rounded-[22px] border border-white/10 bg-gradient-to-br p-4 ${stat.accent}`}>
                  <Icon className={`h-6 w-6 ${stat.tone}`} />
                </div>
              </div>
            </div>
          );
        })}
      </section>

      <section className="grid gap-6 xl:grid-cols-2">
        <MetricsChart data={metricsData} type="area" title="Security events pulse" dataKey="value" color="#4f8cff" showTrend />
        <MetricsChart data={metricsData} type="bar" title="Alerts over time" dataKey="alerts" color="#8b5cf6" />
      </section>

      <section className="grid gap-6 xl:grid-cols-[1.2fr_0.8fr] xl:gap-7">
        <div className="surface-card hover-lift p-7">
          <div className="mb-6 flex items-center justify-between">
            <div>
              <h2 className="text-[1.45rem] font-semibold text-white">Recent alerts</h2>
              <p className="mt-1 text-sm leading-7 text-slate-400">Fresh detections streaming from the runtime pipeline</p>
            </div>
            <Link to="/alerts" className="action-button">
              Open queue
            </Link>
          </div>

          <div className="space-y-3">
            {alerts.slice(0, 5).map((alert) => (
              <div
                key={alert.id}
                className={`rounded-[24px] border p-5 ${
                  alert.severity === 'high'
                    ? 'border-rose-400/18 bg-rose-500/8'
                    : alert.severity === 'medium'
                    ? 'border-amber-400/18 bg-amber-500/8'
                    : 'border-sky-400/18 bg-sky-500/8'
                }`}
              >
                <div className="flex flex-col gap-3 md:flex-row md:items-start md:justify-between">
                  <div>
                    <div className="flex items-center gap-2">
                      <span className="text-sm font-semibold text-white">{alert.kind}</span>
                      <span className="data-badge">{alert.severity}</span>
                    </div>
                    <p className="mt-2 text-sm leading-6 text-slate-300">{alert.message}</p>
                  </div>
                  <p className="text-xs uppercase tracking-[0.18em] text-slate-500">
                    {new Date(alert.createdAt).toLocaleString()}
                  </p>
                </div>
              </div>
            ))}
            {alerts.length === 0 && (
              <div className="rounded-[24px] border border-white/10 bg-slate-950/35 py-14 text-center text-slate-400">
                No alerts at this time
              </div>
            )}
          </div>
        </div>

        <div className="surface-card hover-lift p-7">
          <div className="mb-6">
            <p className="text-xs uppercase tracking-[0.22em] text-slate-500">Cluster snapshot</p>
            <h2 className="mt-2 text-[1.45rem] font-semibold text-white">Workload health overview</h2>
          </div>

          <div className="space-y-4">
            {[
              ['Nodes connected', '03'],
              ['Protected pods', String(activePods)],
              ['Policies pushed', String(activePolicies)],
              ['Pending escalations', String(highSeverityAlerts)],
            ].map(([label, value]) => (
              <div key={label} className="list-card flex items-center justify-between px-5 py-4">
                <span className="text-sm text-slate-400">{label}</span>
                <span className="text-xl font-semibold text-white">{value}</span>
              </div>
            ))}
          </div>
        </div>
      </section>
    </div>
  );
}
