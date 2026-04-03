import { BrowserRouter, Navigate, Outlet, Route, Routes, useLocation } from 'react-router-dom';
import { Toaster } from 'react-hot-toast';
import { Navbar } from '../components/layout/Navbar';
import { apiClient } from '../api/client';
import { useThemeStore } from '../stores/themeStore';
import { Suspense, lazy, useEffect } from 'react';

const Dashboard = lazy(() => import('../views/Dashboard'));
const Policies = lazy(() => import('../views/Policies'));
const Alerts = lazy(() => import('../views/Alerts'));
const Cluster = lazy(() => import('../views/Cluster'));
const Reports = lazy(() => import('../views/Reports'));
const Settings = lazy(() => import('../views/Settings'));
const Login = lazy(() => import('../views/Login'));

function ProtectedRoute() {
  const location = useLocation();
  const token = apiClient.getToken();

  if (!token) {
    return <Navigate to="/login" replace state={{ from: location }} />;
  }

  return (
    <Navbar>
      <Outlet />
    </Navbar>
  );
}

export function App() {
  const { theme } = useThemeStore();

  useEffect(() => {
    // Apply theme to document
    if (theme === 'dark') {
      document.documentElement.classList.add('dark');
    } else {
      document.documentElement.classList.remove('dark');
    }
  }, [theme]);

  return (
    <BrowserRouter>
      <Suspense fallback={<RouteFallback />}>
        <Routes>
          <Route path="/login" element={<Login />} />
          <Route element={<ProtectedRoute />}>
            <Route path="/" element={<Dashboard />} />
            <Route path="/policies" element={<Policies />} />
            <Route path="/alerts" element={<Alerts />} />
            <Route path="/cluster" element={<Cluster />} />
            <Route path="/reports" element={<Reports />} />
            <Route path="/settings" element={<Settings />} />
          </Route>
        </Routes>
      </Suspense>
      <Toaster
        position="top-right"
        toastOptions={{
          duration: 4000,
          style: {
            background: theme === 'dark' ? 'rgba(15, 23, 42, 0.94)' : '#fff',
            color: theme === 'dark' ? '#f8fafc' : '#000',
            border: theme === 'dark' ? '1px solid rgba(96, 165, 250, 0.2)' : '1px solid #e5e7eb',
            borderRadius: '18px',
            boxShadow: theme === 'dark' ? '0 18px 40px rgba(2, 6, 23, 0.35)' : '0 18px 40px rgba(15, 23, 42, 0.08)',
          },
        }}
      />
    </BrowserRouter>
  );
}

function RouteFallback() {
  return (
    <div className="flex min-h-screen items-center justify-center px-4">
      <div className="surface-card-strong w-full max-w-lg p-8 text-center">
        <div className="mx-auto mb-5 h-12 w-12 rounded-2xl border border-sky-400/20 bg-sky-500/10" />
        <p className="text-xs uppercase tracking-[0.24em] text-sky-300/80">Loading workspace</p>
        <h2 className="mt-3 text-2xl font-semibold text-white">Preparing your security console</h2>
        <p className="mt-3 text-sm leading-7 text-slate-400">Fetching the next view and warming up dashboard modules.</p>
      </div>
    </div>
  );
}
