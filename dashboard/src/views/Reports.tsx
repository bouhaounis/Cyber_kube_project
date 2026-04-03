import { useState } from 'react';
import { Download, FileText, Sparkles, TimerReset } from 'lucide-react';
import toast from 'react-hot-toast';
import { MetricsChart } from '../components/charts/MetricsChart';

export default function Reports() {
  const [dateRange, setDateRange] = useState('7d');

  const securityEvents = Array.from({ length: 30 }, (_, i) => ({
    name: `Day ${i + 1}`,
    events: Math.floor(Math.random() * 100),
    blocked: Math.floor(Math.random() * 20),
  }));

  const threatTypes = [
    { name: 'Container Escape', value: 45, color: '#fb7185' },
    { name: 'Network Scan', value: 30, color: '#f59e0b' },
    { name: 'Crypto Mining', value: 15, color: '#38bdf8' },
    { name: 'Privilege Escalation', value: 10, color: '#8b5cf6' },
  ];

  const summaryCards = [
    { label: 'Total events', value: '1,234', note: '+12% from last period', icon: Sparkles, tone: 'text-sky-300' },
    { label: 'Threats blocked', value: '89', note: '+5% from last period', icon: FileText, tone: 'text-emerald-300' },
    { label: 'False positives', value: '12', note: '-3% from last period', icon: TimerReset, tone: 'text-rose-300' },
    { label: 'Response time', value: '0.8s', note: '-15% from last period', icon: Download, tone: 'text-violet-300' },
  ];

  const handleExport = (format: 'pdf' | 'csv') => {
    toast.success(`Exporting report as ${format.toUpperCase()}...`);
  };

  return (
    <div className="page-grid">
      <section className="flex flex-col gap-5 lg:flex-row lg:items-end lg:justify-between">
        <div>
          <div className="status-pill mb-4">Analytics workspace</div>
          <h1 className="section-title">Security reports</h1>
          <p className="mt-2 max-w-2xl text-sm leading-7 text-slate-400">
            Review reporting trends, export operational summaries, and track security outcomes over time.
          </p>
        </div>
        <div className="flex flex-wrap items-center gap-3">
          <select value={dateRange} onChange={(e) => setDateRange(e.target.value)} className="app-input min-w-[180px]">
            <option value="7d">Last 7 days</option>
            <option value="30d">Last 30 days</option>
            <option value="90d">Last 90 days</option>
          </select>
          <button onClick={() => handleExport('pdf')} className="action-button action-button-primary">
            <FileText className="h-4 w-4" />
            Export PDF
          </button>
          <button onClick={() => handleExport('csv')} className="action-button">
            <Download className="h-4 w-4" />
            Export CSV
          </button>
        </div>
      </section>

      <section className="grid gap-4 md:grid-cols-2 xl:grid-cols-4">
        {summaryCards.map(({ label, value, note, icon: MetricIcon, tone }) => {
          return (
            <div key={label} className="metric-card">
              <div className="relative flex items-start justify-between">
                <div>
                  <p className="text-sm text-slate-400">{label}</p>
                  <p className="mt-3 text-4xl font-semibold text-white">{value}</p>
                  <p className={`mt-3 text-sm ${tone}`}>{note}</p>
                </div>
                <div className="rounded-[22px] border border-white/10 bg-white/5 p-3">
                  <MetricIcon className={`h-5 w-5 ${tone}`} />
                </div>
              </div>
            </div>
          );
        })}
      </section>

      <section className="grid gap-6 xl:grid-cols-2">
        <MetricsChart data={securityEvents} type="area" title="Security events over time" dataKey="events" color="#4f8cff" />
        <MetricsChart data={securityEvents} type="bar" title="Threats blocked" dataKey="blocked" color="#8b5cf6" />
      </section>

      <section className="surface-card p-6">
        <h2 className="text-xl font-semibold text-white">Threat type distribution</h2>
        <p className="mt-1 text-sm text-slate-400">Relative concentration of detections across the current reporting range.</p>
        <div className="mt-6 space-y-4">
          {threatTypes.map((threat) => (
            <div key={threat.name} className="rounded-[22px] border border-white/10 bg-slate-950/40 p-4">
              <div className="mb-2 flex items-center justify-between">
                <span className="text-sm font-medium text-white">{threat.name}</span>
                <span className="text-sm text-slate-400">{threat.value}%</span>
              </div>
              <div className="h-2.5 w-full rounded-full bg-slate-800">
                <div className="h-2.5 rounded-full transition-all" style={{ width: `${threat.value}%`, backgroundColor: threat.color }} />
              </div>
            </div>
          ))}
        </div>
      </section>
    </div>
  );
}
