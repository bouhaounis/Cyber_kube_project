import { useMemo, useState } from 'react';
import { Download, FileText, Sparkles, TimerReset } from 'lucide-react';
import jsPDF from 'jspdf';
import toast from 'react-hot-toast';
import { MetricsChart } from '../components/charts/MetricsChart';

const RANGE_IN_DAYS: Record<string, number> = {
  '7d': 7,
  '30d': 30,
  '60d': 60,
  '90d': 90,
};

const BASELINE_EVENTS = Array.from({ length: 90 }, (_, index) => {
  const day = index + 1;
  const events = 54 + (day % 9) * 7 + ((day * 13) % 19);
  const blocked = Math.max(8, Math.round(events * (0.16 + (day % 5) * 0.018)));
  return {
    name: `Day ${day}`,
    events,
    blocked,
  };
});

export default function Reports() {
  const [dateRange, setDateRange] = useState('7d');

  const securityEvents = useMemo(() => {
    const days = RANGE_IN_DAYS[dateRange] ?? 7;
    return BASELINE_EVENTS.slice(-days);
  }, [dateRange]);

  const totalEvents = useMemo(() => securityEvents.reduce((sum, item) => sum + item.events, 0), [securityEvents]);
  const totalBlocked = useMemo(() => securityEvents.reduce((sum, item) => sum + item.blocked, 0), [securityEvents]);
  const falsePositives = useMemo(() => Math.max(4, Math.round(totalEvents * 0.011)), [totalEvents]);
  const responseTime = useMemo(() => (0.62 + securityEvents.length / 180).toFixed(1), [securityEvents.length]);
  const blockedRate = useMemo(() => Math.round((totalBlocked / totalEvents) * 100), [totalBlocked, totalEvents]);

  const threatTypes = useMemo(() => {
    const rangeWeight = securityEvents.length;
    return [
      { name: 'Container Escape', value: Math.round(28 + rangeWeight * 0.18), color: '#fb7185' },
      { name: 'Network Scan', value: Math.round(22 + rangeWeight * 0.12), color: '#f59e0b' },
      { name: 'Crypto Mining', value: Math.round(12 + rangeWeight * 0.07), color: '#38bdf8' },
      { name: 'Privilege Escalation', value: Math.round(8 + rangeWeight * 0.05), color: '#8b5cf6' },
    ];
  }, [securityEvents.length]);

  const summaryCards = useMemo(
    () => [
      { label: 'Total events', value: totalEvents.toLocaleString(), note: `${securityEvents.length}-day reporting window`, icon: Sparkles, tone: 'text-sky-300' },
      { label: 'Threats blocked', value: totalBlocked.toLocaleString(), note: `${blockedRate}% of detected activity`, icon: FileText, tone: 'text-emerald-300' },
      { label: 'False positives', value: falsePositives.toLocaleString(), note: 'Estimated analyst-reviewed noise', icon: TimerReset, tone: 'text-rose-300' },
      { label: 'Response time', value: `${responseTime}s`, note: 'Median policy reaction time', icon: Download, tone: 'text-violet-300' },
    ],
    [blockedRate, falsePositives, responseTime, securityEvents.length, totalBlocked, totalEvents]
  );

  const reportPayload = {
    generatedAt: new Date().toISOString(),
    dateRange,
    summary: summaryCards.map(({ label, value, note }) => ({ label, value, note })),
    threatTypes,
    securityEvents,
  };

  const downloadBlob = (blob: Blob, fileName: string) => {
    const url = URL.createObjectURL(blob);
    const link = document.createElement('a');
    link.href = url;
    link.download = fileName;
    link.click();
    URL.revokeObjectURL(url);
  };

  const handleExport = async (format: 'pdf' | 'json') => {
    try {
      if (format === 'json') {
        downloadBlob(
          new Blob([JSON.stringify(reportPayload, null, 2)], { type: 'application/json' }),
          `cyber-kube-report-${dateRange}.json`
        );
        toast.success('JSON report downloaded');
        return;
      }

      const doc = new jsPDF({
        orientation: 'portrait',
        unit: 'pt',
        format: 'a4',
      });

      const pageWidth = doc.internal.pageSize.getWidth();
      let y = 48;
      doc.setFont('helvetica', 'bold');
      doc.setFontSize(20);
      doc.text('Cyber-Kube Security Report', 40, y);

      y += 22;
      doc.setFont('helvetica', 'normal');
      doc.setFontSize(11);
      doc.text(`Generated: ${new Date(reportPayload.generatedAt).toLocaleString()}`, 40, y);
      y += 16;
      doc.text(`Range: ${dateRange}`, 40, y);

      y += 28;
      doc.setFont('helvetica', 'bold');
      doc.setFontSize(14);
      doc.text('Executive Summary', 40, y);

      y += 18;
      doc.setFont('helvetica', 'normal');
      doc.setFontSize(11);
      for (const card of summaryCards) {
        const line = `${card.label}: ${card.value} (${card.note})`;
        const lines = doc.splitTextToSize(line, pageWidth - 80);
        doc.text(lines, 40, y);
        y += lines.length * 14 + 6;
      }

      y += 10;
      doc.setFont('helvetica', 'bold');
      doc.setFontSize(14);
      doc.text('Threat Distribution', 40, y);

      y += 18;
      doc.setFont('helvetica', 'normal');
      doc.setFontSize(11);
      for (const threat of threatTypes) {
        doc.text(`${threat.name}: ${threat.value}%`, 40, y);
        y += 16;
      }

      y += 10;
      doc.setFont('helvetica', 'bold');
      doc.setFontSize(14);
      doc.text('Event Trend Snapshot', 40, y);

      y += 18;
      doc.setFont('helvetica', 'normal');
      doc.setFontSize(11);
      for (const event of securityEvents.slice(0, 12)) {
        doc.text(`${event.name}: ${event.events} events, ${event.blocked} blocked`, 40, y);
        y += 16;
      }

      doc.save(`cyber-kube-report-${dateRange}.pdf`);
      toast.success('PDF report downloaded');
    } catch (error) {
      const message = error instanceof Error ? error.message : 'Unknown export error';
      toast.error(`Export failed: ${message}`);
    }
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
            <option value="60d">Last 60 days</option>
            <option value="90d">Last 90 days</option>
          </select>
          <button onClick={() => handleExport('pdf')} className="action-button action-button-primary">
            <FileText className="h-4 w-4" />
            Export PDF
          </button>
          <button onClick={() => handleExport('json')} className="action-button">
            <Download className="h-4 w-4" />
            Export JSON
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
