import { useState } from 'react';
import { AlertTriangle, ShieldAlert, Siren, Sparkles } from 'lucide-react';
import { useAlertStore } from '../stores/alertStore';
import { DataTable } from '../components/tables/DataTable';

export default function Alerts() {
  const { alerts, clearAlerts } = useAlertStore();
  const [severityFilter, setSeverityFilter] = useState<string>('all');

  const filteredAlerts = severityFilter === 'all' ? alerts : alerts.filter((a) => a.severity === severityFilter);

  const columns = [
    {
      key: 'kind',
      header: 'Signal',
      sortable: true,
      render: (alert: any) => (
        <div className="flex items-center gap-3">
          <div className="rounded-2xl border border-rose-400/15 bg-rose-500/10 p-2">
            <AlertTriangle className="h-4 w-4 text-rose-300" />
          </div>
          <div>
            <p className="font-semibold text-white">{alert.kind}</p>
            <p className="text-xs uppercase tracking-[0.18em] text-slate-500">Runtime detection</p>
          </div>
        </div>
      ),
    },
    {
      key: 'severity',
      header: 'Severity',
      sortable: true,
      render: (alert: any) => (
        <span
          className={`data-badge ${
            alert.severity === 'high'
              ? 'border-rose-400/20 bg-rose-500/10 text-rose-200'
              : alert.severity === 'medium'
              ? 'border-amber-400/20 bg-amber-500/10 text-amber-200'
              : 'border-sky-400/20 bg-sky-500/10 text-sky-200'
          }`}
        >
          {alert.severity}
        </span>
      ),
    },
    {
      key: 'message',
      header: 'Message',
      sortable: true,
      render: (alert: any) => <span className="text-slate-300">{alert.message}</span>,
    },
    {
      key: 'createdAt',
      header: 'Timestamp',
      sortable: true,
      render: (alert: any) => (
        <span className="font-mono text-xs text-slate-400">{new Date(alert.createdAt).toLocaleString()}</span>
      ),
    },
  ];

  const severityCounts = {
    all: alerts.length,
    high: alerts.filter((a) => a.severity === 'high').length,
    medium: alerts.filter((a) => a.severity === 'medium').length,
    low: alerts.filter((a) => a.severity === 'low').length,
  };

  const cards = [
    { key: 'all', label: 'All signals', icon: Sparkles, tone: 'text-sky-300' },
    { key: 'high', label: 'Critical', icon: Siren, tone: 'text-rose-300' },
    { key: 'medium', label: 'Elevated', icon: ShieldAlert, tone: 'text-amber-300' },
    { key: 'low', label: 'Informational', icon: AlertTriangle, tone: 'text-cyan-300' },
  ];

  return (
    <div className="page-grid">
      <section className="flex flex-col gap-5 lg:flex-row lg:items-end lg:justify-between">
        <div>
          <div className="status-pill mb-4">Incident queue</div>
          <h1 className="section-title">Security alerts</h1>
          <p className="mt-2 max-w-2xl text-sm leading-7 text-slate-400">
            Review detections, isolate high-severity runtime events, and keep the investigation stream organized.
          </p>
        </div>
        {alerts.length > 0 && (
          <button
            onClick={() => {
              if (window.confirm('Clear all alerts?')) {
                clearAlerts();
              }
            }}
            className="action-button"
          >
            Clear alert queue
          </button>
        )}
      </section>

      <section className="grid gap-4 md:grid-cols-2 xl:grid-cols-4">
        {cards.map((card) => {
          const Icon = card.icon;
          const active = severityFilter === card.key;

          return (
            <button
              key={card.key}
              onClick={() => setSeverityFilter(card.key)}
              className={`metric-card cursor-pointer text-left ${
                active ? 'ring-1 ring-sky-400/35' : 'hover:border-sky-400/18'
              }`}
            >
              <div className="relative flex items-start justify-between">
                <div>
                  <p className="text-sm text-slate-400">{card.label}</p>
                  <p className="mt-3 text-4xl font-semibold text-white">
                    {severityCounts[card.key as keyof typeof severityCounts]}
                  </p>
                </div>
                <div className="rounded-[20px] border border-white/10 bg-white/5 p-3">
                  <Icon className={`h-5 w-5 ${card.tone}`} />
                </div>
              </div>
            </button>
          );
        })}
      </section>

      <section>
        <DataTable data={filteredAlerts} columns={columns} searchable pagination pageSize={15} />
      </section>
    </div>
  );
}
