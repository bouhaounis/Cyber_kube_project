import {
  Area,
  AreaChart,
  Bar,
  BarChart,
  CartesianGrid,
  Legend,
  Line,
  LineChart,
  ResponsiveContainer,
  Tooltip,
  XAxis,
  YAxis,
} from 'recharts';
import { TrendingDown, TrendingUp } from 'lucide-react';

type MetricRecord = {
  name: string;
  value?: number;
  [key: string]: string | number | undefined;
};

interface MetricsChartProps {
  data: MetricRecord[];
  type?: 'line' | 'area' | 'bar';
  title?: string;
  dataKey?: string;
  color?: string;
  showTrend?: boolean;
}

type TooltipValue = string | number | Array<string | number>;

function ChartTooltip({
  active,
  payload,
  label,
}: {
  active?: boolean;
  payload?: Array<{ name?: string; value?: TooltipValue; color?: string }>;
  label?: string;
}) {
  if (!active || !payload?.length) {
    return null;
  }

  return (
    <div className="rounded-2xl border border-sky-400/15 bg-slate-950/95 px-4 py-3 shadow-2xl backdrop-blur-xl">
      <p className="mb-2 text-xs uppercase tracking-[0.22em] text-slate-500">{label}</p>
      <div className="space-y-2">
        {payload.map((entry) => (
          <div key={`${entry.name}-${entry.color}`} className="flex items-center justify-between gap-8 text-sm">
            <span className="flex items-center gap-2 text-slate-300">
              <span className="h-2.5 w-2.5 rounded-full" style={{ backgroundColor: entry.color }} />
              {entry.name}
            </span>
            <span className="font-semibold text-white">{entry.value}</span>
          </div>
        ))}
      </div>
    </div>
  );
}

export function MetricsChart({
  data,
  type = 'line',
  title,
  dataKey = 'value',
  color = '#4f8cff',
  showTrend = false,
}: MetricsChartProps) {
  const firstValue = Number(data[0]?.[dataKey] ?? 0);
  const lastValue = Number(data[data.length - 1]?.[dataKey] ?? 0);
  const trend = data.length > 1 ? lastValue - firstValue : 0;
  const isPositive = trend >= 0;

  return (
    <div className="surface-card hover-lift h-full p-5 md:p-6">
      {title && (
        <div className="mb-5 flex items-start justify-between gap-3">
          <div>
            <h3 className="text-lg font-semibold text-white">{title}</h3>
            <p className="mt-1 text-sm text-slate-400">Live security telemetry across the selected window</p>
          </div>
          {showTrend && (
            <div
              className={`inline-flex items-center gap-1 rounded-full px-3 py-1 text-xs font-semibold ${
                isPositive ? 'bg-emerald-500/10 text-emerald-300' : 'bg-rose-500/10 text-rose-300'
              }`}
            >
              {isPositive ? <TrendingUp className="h-4 w-4" /> : <TrendingDown className="h-4 w-4" />}
              {isPositive ? '+' : ''}
              {trend.toFixed(1)}
            </div>
          )}
        </div>
      )}

      <ResponsiveContainer width="100%" height={300}>
        {type === 'area' ? (
          <AreaChart data={data}>
            <defs>
              <linearGradient id={`gradient-${dataKey}`} x1="0" y1="0" x2="0" y2="1">
                <stop offset="5%" stopColor={color} stopOpacity={0.36} />
                <stop offset="95%" stopColor={color} stopOpacity={0.04} />
              </linearGradient>
            </defs>
            <CartesianGrid stroke="rgba(148, 163, 184, 0.12)" strokeDasharray="3 3" vertical={false} />
            <XAxis dataKey="name" tick={{ fill: '#64748b', fontSize: 12 }} axisLine={false} tickLine={false} />
            <YAxis tick={{ fill: '#64748b', fontSize: 12 }} axisLine={false} tickLine={false} />
            <Tooltip content={<ChartTooltip />} />
            <Legend wrapperStyle={{ color: '#94a3b8', fontSize: '12px', paddingTop: '10px' }} />
            <Area
              type="monotone"
              dataKey={dataKey}
              stroke={color}
              strokeWidth={2.5}
              fill={`url(#gradient-${dataKey})`}
              name={title ?? dataKey}
            />
          </AreaChart>
        ) : type === 'bar' ? (
          <BarChart data={data}>
            <CartesianGrid stroke="rgba(148, 163, 184, 0.12)" strokeDasharray="3 3" vertical={false} />
            <XAxis dataKey="name" tick={{ fill: '#64748b', fontSize: 12 }} axisLine={false} tickLine={false} />
            <YAxis tick={{ fill: '#64748b', fontSize: 12 }} axisLine={false} tickLine={false} />
            <Tooltip content={<ChartTooltip />} />
            <Legend wrapperStyle={{ color: '#94a3b8', fontSize: '12px', paddingTop: '10px' }} />
            <Bar dataKey={dataKey} fill={color} radius={[10, 10, 4, 4]} name={title ?? dataKey} />
          </BarChart>
        ) : (
          <LineChart data={data}>
            <CartesianGrid stroke="rgba(148, 163, 184, 0.12)" strokeDasharray="3 3" vertical={false} />
            <XAxis dataKey="name" tick={{ fill: '#64748b', fontSize: 12 }} axisLine={false} tickLine={false} />
            <YAxis tick={{ fill: '#64748b', fontSize: 12 }} axisLine={false} tickLine={false} />
            <Tooltip content={<ChartTooltip />} />
            <Legend wrapperStyle={{ color: '#94a3b8', fontSize: '12px', paddingTop: '10px' }} />
            <Line
              type="monotone"
              dataKey={dataKey}
              stroke={color}
              strokeWidth={2.5}
              dot={false}
              activeDot={{ r: 5, fill: color, stroke: '#020617', strokeWidth: 2 }}
              name={title ?? dataKey}
            />
          </LineChart>
        )}
      </ResponsiveContainer>
    </div>
  );
}
