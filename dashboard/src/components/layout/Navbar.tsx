import { ReactNode, useEffect, useMemo, useState } from 'react';
import { Link, useLocation, useNavigate } from 'react-router-dom';
import {
  AlertTriangle,
  Bell,
  ChevronRight,
  Command,
  LayoutDashboard,
  LogOut,
  Menu,
  Moon,
  Network,
  PanelLeftClose,
  PanelLeftOpen,
  Radar,
  FileText,
  Settings,
  Shield,
  SunMedium,
  X,
} from 'lucide-react';
import toast from 'react-hot-toast';
import { useThemeStore } from '../../stores/themeStore';
import { apiClient } from '../../api/client';

const navigation = [
  { name: 'Dashboard', href: '/', icon: LayoutDashboard, description: 'Live threat overview' },
  { name: 'Policies', href: '/policies', icon: Shield, description: 'Zero-trust controls' },
  { name: 'Alerts', href: '/alerts', icon: AlertTriangle, description: 'Incidents and detections' },
  { name: 'Cluster', href: '/cluster', icon: Network, description: 'Runtime topology' },
  { name: 'Reports', href: '/reports', icon: FileText, description: 'Analytics and exports' },
  { name: 'Settings', href: '/settings', icon: Settings, description: 'Platform preferences' },
];

function pageTitle(pathname: string): string {
  return navigation.find((item) => item.href === pathname)?.name ?? 'Dashboard';
}

export function Navbar({ children }: { children: ReactNode }) {
  const location = useLocation();
  const navigate = useNavigate();
  const [mobileMenuOpen, setMobileMenuOpen] = useState(false);
  const [collapsed, setCollapsed] = useState(false);
  const [commandOpen, setCommandOpen] = useState(false);
  const [notificationsOpen, setNotificationsOpen] = useState(false);
  const { theme, toggleTheme } = useThemeStore();

  const notifications = [
    { id: 'n-1', title: 'Engine heartbeat healthy', detail: 'All policy workers are responding normally.', tone: 'text-emerald-300' },
    { id: 'n-2', title: '3 alerts need review', detail: 'Recent runtime detections are waiting in the alert queue.', tone: 'text-amber-300' },
    { id: 'n-3', title: 'Metrics export ready', detail: 'Reports can now be exported as PDF and JSON.', tone: 'text-sky-300' },
  ];

  const commandItems = [
    { label: 'Open dashboard', description: 'Jump to the overview workspace', href: '/' },
    { label: 'Create policy', description: 'Open the policy management view', href: '/policies' },
    { label: 'Review alerts', description: 'Inspect active detections and incidents', href: '/alerts' },
    { label: 'Export reports', description: 'Open analytics and report downloads', href: '/reports' },
    { label: 'Open settings', description: 'Adjust API and theme preferences', href: '/settings' },
  ];

  const activeItem = useMemo(
    () => navigation.find((item) => item.href === location.pathname) ?? navigation[0],
    [location.pathname]
  );

  useEffect(() => {
    const handleKeyDown = (event: KeyboardEvent) => {
      if ((event.ctrlKey || event.metaKey) && event.key.toLowerCase() === 'k') {
        event.preventDefault();
        setCommandOpen((value) => !value);
        setNotificationsOpen(false);
      }

      if (event.key === 'Escape') {
        setCommandOpen(false);
        setNotificationsOpen(false);
      }
    };

    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, []);

  const handleSignOut = () => {
    apiClient.clearToken();
    window.location.assign('/login');
  };

  const handleCommandNavigate = (href: string) => {
    setCommandOpen(false);
    navigate(href);
  };

  const handleNotificationOpen = () => {
    setNotificationsOpen((value) => !value);
    setCommandOpen(false);
  };

  const markNotificationsRead = () => {
    setNotificationsOpen(false);
    toast.success('Notifications cleared');
  };

  return (
    <div className="app-shell">
      <div className="flex min-h-screen">
        <aside
          className={`shell-panel-strong fixed inset-y-0 left-0 z-40 hidden border-r lg:flex lg:flex-col ${
            collapsed ? 'lg:w-[96px]' : 'lg:w-[296px]'
          }`}
          style={{ background: 'var(--bg-sidebar)' }}
        >
          <div className="flex items-center justify-between px-5 py-5">
            <div className={`flex items-center gap-3 ${collapsed ? 'justify-center' : ''}`}>
              <div className="flex h-12 w-12 items-center justify-center rounded-2xl border border-sky-400/30 bg-gradient-to-br from-sky-500/30 via-blue-500/20 to-violet-500/25 shadow-[0_18px_45px_rgba(59,130,246,0.18)]">
                <Shield className="h-6 w-6 text-sky-300" />
              </div>
              {!collapsed && (
                <div>
                  <p className="text-[11px] font-semibold uppercase tracking-[0.28em] text-sky-300/80">
                    K-SOC Command
                  </p>
                  <h1 className="text-xl font-semibold text-slate-50">Cyber-Kube</h1>
                </div>
              )}
            </div>
            <button
              type="button"
              onClick={() => setCollapsed((value) => !value)}
              className="hidden cursor-pointer rounded-2xl border border-white/10 bg-white/5 p-2 text-slate-300 hover:border-sky-400/30 hover:bg-slate-800/80 lg:inline-flex"
              aria-label={collapsed ? 'Expand sidebar' : 'Collapse sidebar'}
            >
              {collapsed ? <PanelLeftOpen className="h-4 w-4" /> : <PanelLeftClose className="h-4 w-4" />}
            </button>
          </div>

          <div className="px-4">
            <div className="rounded-[26px] border border-sky-400/10 bg-slate-950/50 px-4 py-4">
              <div className={`flex items-center gap-3 ${collapsed ? 'justify-center' : ''}`}>
                <div className="relative">
                  <div className="flex h-10 w-10 items-center justify-center rounded-2xl bg-gradient-to-br from-emerald-400/20 to-cyan-400/20">
                    <Radar className="h-5 w-5 text-emerald-300" />
                  </div>
                  <div className="absolute bottom-0 right-0 h-3.5 w-3.5 rounded-full border-2 border-slate-950 bg-emerald-400" />
                </div>
                {!collapsed && (
                  <div>
                    <p className="text-sm font-semibold text-slate-100">Protection Active</p>
                    <p className="text-xs text-slate-400">Runtime policies synced</p>
                  </div>
                )}
              </div>
            </div>
          </div>

          <nav className="mt-6 flex-1 space-y-2 px-4">
            {navigation.map((item) => {
              const Icon = item.icon;
              const isActive = location.pathname === item.href;

              return (
                <Link
                  key={item.name}
                  to={item.href}
                  className={`nav-link group flex cursor-pointer items-center gap-3 rounded-[22px] border px-4 py-3.5 ${
                    isActive
                      ? 'border-sky-400/35 bg-gradient-to-r from-sky-500/18 to-violet-500/14 text-white shadow-[0_18px_50px_rgba(59,130,246,0.14)]'
                      : 'border-transparent bg-transparent text-slate-400 hover:border-white/10 hover:bg-slate-900/80 hover:text-slate-100'
                  }`}
                >
                  <span
                    className={`icon-chip flex h-11 w-11 shrink-0 items-center justify-center rounded-2xl ${
                      isActive ? 'bg-sky-500/18 text-sky-200' : 'bg-white/5 text-slate-400 group-hover:text-slate-100'
                    }`}
                  >
                    <Icon className="h-5 w-5" />
                  </span>
                  {!collapsed && (
                    <span className="min-w-0">
                      <span className="block text-sm font-semibold">{item.name}</span>
                      <span className="block truncate text-xs text-slate-500">{item.description}</span>
                    </span>
                  )}
                </Link>
              );
            })}
          </nav>

          <div className="px-4 pb-4">
            <button
              type="button"
              onClick={handleSignOut}
              className="flex w-full cursor-pointer items-center gap-3 rounded-[22px] border border-white/10 bg-white/5 px-4 py-3 text-sm font-medium text-slate-300 hover:border-rose-400/25 hover:bg-rose-500/10 hover:text-white"
            >
              <span className="flex h-10 w-10 items-center justify-center rounded-2xl bg-white/5">
                <LogOut className="h-4 w-4" />
              </span>
              {!collapsed && <span>Sign out</span>}
            </button>
          </div>
        </aside>

        {mobileMenuOpen && (
          <div className="fixed inset-0 z-50 bg-slate-950/80 backdrop-blur-sm lg:hidden">
            <div className="absolute inset-y-0 left-0 w-[86%] max-w-[320px] border-r border-white/10 bg-[rgba(5,10,26,0.96)] px-4 py-5 shadow-2xl">
              <div className="mb-6 flex items-center justify-between">
                <div className="flex items-center gap-3">
                  <div className="flex h-11 w-11 items-center justify-center rounded-2xl bg-gradient-to-br from-sky-500/35 to-violet-500/30">
                    <Shield className="h-5 w-5 text-sky-100" />
                  </div>
                  <div>
                    <p className="text-xs uppercase tracking-[0.24em] text-sky-300/70">K-SOC</p>
                    <p className="text-lg font-semibold text-white">Control Center</p>
                  </div>
                </div>
                <button
                  type="button"
                  onClick={() => setMobileMenuOpen(false)}
                  className="rounded-2xl border border-white/10 bg-white/5 p-2 text-slate-300"
                >
                  <X className="h-5 w-5" />
                </button>
              </div>

              <div className="space-y-2">
                {navigation.map((item) => {
                  const Icon = item.icon;
                  const isActive = location.pathname === item.href;

                  return (
                    <Link
                      key={item.name}
                      to={item.href}
                      onClick={() => setMobileMenuOpen(false)}
                      className={`nav-link flex items-center gap-3 rounded-[22px] border px-4 py-3 ${
                        isActive
                          ? 'border-sky-400/35 bg-gradient-to-r from-sky-500/18 to-violet-500/14 text-white'
                          : 'border-transparent text-slate-300 hover:border-white/10 hover:bg-slate-900/70'
                      }`}
                    >
                      <Icon className="h-5 w-5" />
                      <span>{item.name}</span>
                    </Link>
                  );
                })}
              </div>
            </div>
          </div>
        )}

        <div className={`flex min-h-screen flex-1 flex-col ${collapsed ? 'lg:pl-[96px]' : 'lg:pl-[296px]'}`}>
          <header className="sticky top-0 z-30 px-4 pt-4 md:px-6 md:pt-5">
            <div className="surface-card-strong content-frame flex min-h-[92px] items-center justify-between gap-4 px-5 py-4 md:px-7">
              <div className="flex min-w-0 items-center gap-3">
                <button
                  type="button"
                  onClick={() => setMobileMenuOpen(true)}
                  className="topbar-button inline-flex cursor-pointer items-center justify-center rounded-2xl border border-white/10 bg-white/5 p-2.5 text-slate-200 lg:hidden"
                >
                  <Menu className="h-5 w-5" />
                </button>
                <div className="min-w-0">
                  <div className="mb-1.5 flex items-center gap-2 text-[11px] uppercase tracking-[0.24em] text-slate-500">
                    <span>Security Operations</span>
                    <ChevronRight className="h-3.5 w-3.5" />
                    <span>{pageTitle(location.pathname)}</span>
                  </div>
                  <h2 className="truncate text-[1.7rem] font-semibold tracking-tight text-white">{activeItem.name}</h2>
                </div>
              </div>

              <div className="flex items-center gap-2 md:gap-3">
                <div className="hidden items-center gap-2 rounded-2xl border border-emerald-400/20 bg-emerald-500/10 px-3 py-2 text-sm text-emerald-300 md:flex">
                  <span className="h-2.5 w-2.5 rounded-full bg-emerald-400 shadow-[0_0_18px_rgba(74,222,128,0.7)]" />
                  Monitoring healthy
                </div>
                <button
                  type="button"
                  onClick={() => {
                    setCommandOpen(true);
                    setNotificationsOpen(false);
                  }}
                  className="topbar-button hidden cursor-pointer items-center gap-2 rounded-2xl border border-white/10 bg-white/5 px-3 py-2 text-sm text-slate-300 hover:bg-slate-900/80 md:inline-flex"
                >
                  <Command className="h-4 w-4" />
                  Cmd
                </button>
                <button
                  type="button"
                  onClick={handleNotificationOpen}
                  className="topbar-button relative cursor-pointer rounded-2xl border border-white/10 bg-white/5 p-2.5 text-slate-300 hover:bg-slate-900/80"
                >
                  <Bell className="h-5 w-5" />
                  <span className="absolute right-2 top-2 h-2 w-2 rounded-full bg-rose-400" />
                </button>
                <button
                  type="button"
                  onClick={toggleTheme}
                  className="topbar-button cursor-pointer rounded-2xl border border-white/10 bg-white/5 p-2.5 text-slate-300 hover:bg-slate-900/80"
                  aria-label="Toggle theme"
                >
                  {theme === 'dark' ? <SunMedium className="h-5 w-5" /> : <Moon className="h-5 w-5" />}
                </button>
              </div>
            </div>
          </header>

          <main className="flex-1 px-4 pb-8 pt-7 md:px-6 md:pb-10 md:pt-8">
            <div className="content-frame">{children}</div>
          </main>
        </div>
      </div>

      {commandOpen && (
        <div className="fixed inset-0 z-[70] flex items-start justify-center bg-slate-950/55 px-4 pt-24 backdrop-blur-sm" onClick={() => setCommandOpen(false)}>
          <div
            className="surface-card-strong w-full max-w-2xl p-5 md:p-6"
            onClick={(event) => event.stopPropagation()}
          >
            <div className="mb-5 flex items-center justify-between gap-4">
              <div>
                <p className="text-xs uppercase tracking-[0.24em] text-sky-300/75">Quick command</p>
                <h3 className="mt-2 text-2xl font-semibold text-white">Jump through the workspace</h3>
              </div>
              <button
                type="button"
                onClick={() => setCommandOpen(false)}
                className="topbar-button rounded-2xl border border-white/10 bg-white/5 p-2.5 text-slate-300"
              >
                <X className="h-4 w-4" />
              </button>
            </div>

            <div className="space-y-3">
              {commandItems.map((item) => (
                <button
                  key={item.href}
                  type="button"
                  onClick={() => handleCommandNavigate(item.href)}
                  className="list-card flex w-full items-center justify-between gap-4 text-left"
                >
                  <div>
                    <p className="text-sm font-semibold text-white">{item.label}</p>
                    <p className="mt-1 text-sm text-slate-400">{item.description}</p>
                  </div>
                  <ChevronRight className="h-4 w-4 text-slate-400" />
                </button>
              ))}
            </div>
          </div>
        </div>
      )}

      {notificationsOpen && (
        <div className="fixed right-4 top-24 z-[65] w-[min(92vw,380px)] md:right-6">
          <div className="surface-card-strong p-5">
            <div className="mb-4 flex items-center justify-between gap-3">
              <div>
                <p className="text-xs uppercase tracking-[0.24em] text-sky-300/75">Notifications</p>
                <h3 className="mt-1 text-xl font-semibold text-white">Operations inbox</h3>
              </div>
              <button type="button" onClick={markNotificationsRead} className="action-button px-3 py-2 text-xs">
                Clear
              </button>
            </div>

            <div className="space-y-3">
              {notifications.map((item) => (
                <div key={item.id} className="list-card">
                  <div className="flex items-start gap-3">
                    <span className={`mt-1 h-2.5 w-2.5 rounded-full bg-current ${item.tone}`} />
                    <div>
                      <p className="text-sm font-semibold text-white">{item.title}</p>
                      <p className="mt-1 text-sm leading-6 text-slate-400">{item.detail}</p>
                    </div>
                  </div>
                </div>
              ))}
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
