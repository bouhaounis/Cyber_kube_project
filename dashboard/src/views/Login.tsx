import { FormEvent, useState } from 'react';
import { useLocation, useNavigate } from 'react-router-dom';
import toast from 'react-hot-toast';
import { Lock, Shield, User } from 'lucide-react';
import { apiClient } from '../api/client';
import { useThemeStore } from '../stores/themeStore';

type LocationState = {
  from?: {
    pathname?: string;
  };
};

export default function Login() {
  const navigate = useNavigate();
  const location = useLocation();
  const { theme } = useThemeStore();
  const [username, setUsername] = useState('admin');
  const [password, setPassword] = useState('');
  const [submitting, setSubmitting] = useState(false);

  const state = location.state as LocationState | null;
  const redirectTo = state?.from?.pathname || '/';

  async function handleSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    setSubmitting(true);

    try {
      await apiClient.login(username, password);
      toast.success('Login successful');
      navigate(redirectTo, { replace: true });
    } catch (error) {
      const message = error instanceof Error ? error.message : 'Login failed';
      toast.error(message);
    } finally {
      setSubmitting(false);
    }
  }

  return (
    <div className="relative flex min-h-screen items-center justify-center overflow-hidden px-4 py-10">
      <div
        className="absolute inset-0"
        style={{
          background:
            theme === 'dark'
              ? 'radial-gradient(circle at top left, rgba(59,130,246,0.25), transparent 30%), radial-gradient(circle at 80% 20%, rgba(139,92,246,0.24), transparent 30%), linear-gradient(180deg, #040816 0%, #020617 100%)'
              : 'radial-gradient(circle at top left, rgba(59,130,246,0.14), transparent 30%), radial-gradient(circle at 80% 20%, rgba(139,92,246,0.12), transparent 30%), linear-gradient(180deg, #f6f9ff 0%, #eaf2ff 100%)',
        }}
      />
      <div
        className="absolute inset-0 bg-[size:56px_56px]"
        style={{
          backgroundImage:
            theme === 'dark'
              ? 'linear-gradient(90deg, rgba(255,255,255,0.02) 1px, transparent 1px), linear-gradient(rgba(255,255,255,0.02) 1px, transparent 1px)'
              : 'linear-gradient(90deg, rgba(15,23,42,0.04) 1px, transparent 1px), linear-gradient(rgba(15,23,42,0.04) 1px, transparent 1px)',
          opacity: theme === 'dark' ? 0.5 : 0.75,
        }}
      />

      <div className="relative grid w-full max-w-6xl gap-6 lg:grid-cols-[1.05fr_0.95fr]">
        <div className="surface-card-strong hidden p-10 lg:block">
          <div className="status-pill mb-6">Operator access</div>
          <h1 className="text-5xl font-semibold leading-tight text-white">
            Secure control for runtime defense operations.
          </h1>
          <p className="mt-5 max-w-xl text-base leading-8 text-slate-300">
            K-SOC consolidates live detections, policy controls, and cluster telemetry into a single observability workspace for high-speed incident response.
          </p>

          <div className="mt-10 grid gap-4">
            {[
              ['Live event streaming', 'Real-time alerts and engine activity'],
              ['Policy enforcement visibility', 'Understand what is blocked, flagged, or drifting'],
              ['Kubernetes-aware telemetry', 'Namespace and workload context on every critical signal'],
            ].map(([title, body]) => (
              <div key={title} className="rounded-[24px] border border-white/10 bg-slate-950/45 p-5">
                <p className="text-lg font-semibold text-white">{title}</p>
                <p className="mt-2 text-sm leading-7 text-slate-400">{body}</p>
              </div>
            ))}
          </div>
        </div>

        <div className="surface-card-strong w-full p-7 sm:p-9">
          <div className="mb-8 flex items-center gap-4">
            <div className="flex h-14 w-14 items-center justify-center rounded-[22px] border border-sky-400/30 bg-gradient-to-br from-sky-500/30 to-violet-500/25 shadow-[0_18px_45px_rgba(59,130,246,0.18)]">
              <Shield className="h-7 w-7 text-sky-100" />
            </div>
            <div>
              <p className="text-xs uppercase tracking-[0.26em] text-sky-300/75">K-SOC</p>
              <h2 className="text-3xl font-semibold text-white">Operator login</h2>
            </div>
          </div>

          <form className="space-y-5" onSubmit={handleSubmit}>
            <div>
              <label htmlFor="username" className="mb-2 block text-sm font-medium text-slate-300">
                Username
              </label>
              <div className="relative">
                <User className="pointer-events-none absolute left-4 top-1/2 h-4 w-4 -translate-y-1/2 text-slate-500" />
                <input
                  id="username"
                  type="text"
                  value={username}
                  onChange={(event) => setUsername(event.target.value)}
                  className="app-input pl-11"
                  autoComplete="username"
                  required
                />
              </div>
            </div>

            <div>
              <label htmlFor="password" className="mb-2 block text-sm font-medium text-slate-300">
                Password
              </label>
              <div className="relative">
                <Lock className="pointer-events-none absolute left-4 top-1/2 h-4 w-4 -translate-y-1/2 text-slate-500" />
                <input
                  id="password"
                  type="password"
                  value={password}
                  onChange={(event) => setPassword(event.target.value)}
                  className="app-input pl-11"
                  autoComplete="current-password"
                  required
                />
              </div>
            </div>

            <button type="submit" disabled={submitting} className="action-button action-button-primary w-full justify-center py-3">
              {submitting ? 'Signing in...' : 'Sign in to dashboard'}
            </button>
          </form>

          <div className="mt-8 rounded-[24px] border border-white/10 bg-slate-950/45 p-4">
            <p className="text-xs uppercase tracking-[0.22em] text-slate-500">Access note</p>
            <p className="mt-2 text-sm leading-7 text-slate-400">
              Your session token is stored locally and used automatically for protected API requests until you sign out or receive a 401 response.
            </p>
          </div>
        </div>
      </div>
    </div>
  );
}
