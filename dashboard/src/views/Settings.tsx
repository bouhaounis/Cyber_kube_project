import { ReactNode, useEffect, useState } from 'react';
import { Bell, Database, MailCheck, Network, Save, Shield } from 'lucide-react';
import toast from 'react-hot-toast';
import { useThemeStore } from '../stores/themeStore';
import { apiClient } from '../api/client';
import type { AppSettings } from '../api/types';

export default function Settings() {
  const { theme, setTheme } = useThemeStore();
  const [settings, setSettings] = useState({
    notifications: false,
    notificationEmail: '',
    autoRemediation: true,
    logRetention: '30',
    apiUrl: apiClient.getApiUrl(),
  });
  const [saving, setSaving] = useState(false);
  const [testingEmail, setTestingEmail] = useState(false);

  useEffect(() => {
    void loadSettings();
  }, []);

  const loadSettings = async () => {
    try {
      const data = await apiClient.get<AppSettings>('/api/v1/settings');
      setSettings((current) => ({
        ...current,
        notifications: data.email_notifications,
        notificationEmail: data.notification_email,
        autoRemediation: data.auto_remediation,
        logRetention: String(data.log_retention_days),
      }));
    } catch (error) {
      console.error('Failed to load settings', error);
    }
  };

  const handleSave = async () => {
    setSaving(true);
    try {
      await saveSettings();
      apiClient.setApiUrl(settings.apiUrl);
      toast.success('Settings saved successfully');
    } catch (error) {
      const message = error instanceof Error ? error.message : 'Failed to save settings';
      toast.error(message);
    } finally {
      setSaving(false);
    }
  };

  const saveSettings = async () => {
    return apiClient.put<AppSettings>('/api/v1/settings', {
      email_notifications: settings.notifications,
      notification_email: settings.notificationEmail,
      auto_remediation: settings.autoRemediation,
      log_retention_days: Number(settings.logRetention) || 30,
    });
  };

  const handleTestEmail = async () => {
    setTestingEmail(true);
    try {
      await saveSettings();
      await apiClient.post<{ message: string; to: string }>('/api/v1/settings/test-email', {});
      toast.success('Test email sent');
    } catch (error) {
      const message = error instanceof Error ? error.message : 'Failed to send test email';
      toast.error(message);
    } finally {
      setTestingEmail(false);
    }
  };

  return (
    <div className="page-grid">
      <section>
        <div className="status-pill mb-4">Configuration center</div>
        <h1 className="section-title">Settings</h1>
        <p className="mt-2 max-w-2xl text-sm leading-7 text-slate-400">
          Configure presentation, notification behavior, storage defaults, and platform endpoints without changing runtime logic.
        </p>
      </section>

      <section className="grid gap-6 xl:grid-cols-2">
        <SettingsCard icon={Shield} title="Appearance" description="Theme and operator experience">
          <div>
            <label className="mb-2 block text-sm font-medium text-slate-300">Theme</label>
            <select value={theme} onChange={(e) => setTheme(e.target.value as 'light' | 'dark')} className="app-input">
              <option value="light">Light</option>
              <option value="dark">Dark</option>
            </select>
          </div>
        </SettingsCard>

        <SettingsCard icon={Bell} title="Notifications" description="Alert delivery preferences">
          <div className="space-y-3">
            <ToggleRow
              label="Enable email notifications"
              checked={settings.notifications}
              onChange={(checked) => setSettings({ ...settings, notifications: checked })}
            />
            <div>
              <label className="mb-2 block text-sm font-medium text-slate-300">Notification email</label>
              <input
                type="email"
                value={settings.notificationEmail}
                onChange={(e) => setSettings({ ...settings, notificationEmail: e.target.value })}
                className="app-input"
                placeholder="soc-team@example.com"
              />
            </div>
            <button
              type="button"
              onClick={() => void handleTestEmail()}
              disabled={testingEmail || !settings.notifications}
              className="action-button w-full"
            >
              <MailCheck className="h-4 w-4" />
              {testingEmail ? 'Sending test...' : 'Send test email'}
            </button>
            <ToggleRow
              label="Auto-remediation on threats"
              checked={settings.autoRemediation}
              onChange={(checked) => setSettings({ ...settings, autoRemediation: checked })}
            />
          </div>
        </SettingsCard>

        <SettingsCard icon={Database} title="Data & storage" description="Retention and storage posture">
          <div>
            <label className="mb-2 block text-sm font-medium text-slate-300">Log retention (days)</label>
            <input
              type="number"
              value={settings.logRetention}
              onChange={(e) => setSettings({ ...settings, logRetention: e.target.value })}
              className="app-input"
            />
          </div>
        </SettingsCard>

        <SettingsCard icon={Network} title="API configuration" description="Engine and dashboard connectivity">
          <div>
            <label className="mb-2 block text-sm font-medium text-slate-300">API URL</label>
            <input
              type="text"
              value={settings.apiUrl}
              onChange={(e) => setSettings({ ...settings, apiUrl: e.target.value })}
              className="app-input"
            />
          </div>
        </SettingsCard>
      </section>

      <section className="flex justify-end">
        <button onClick={() => void handleSave()} disabled={saving} className="action-button action-button-primary">
          <Save className="h-4 w-4" />
          {saving ? 'Saving...' : 'Save settings'}
        </button>
      </section>
    </div>
  );
}

function SettingsCard({
  icon: Icon,
  title,
  description,
  children,
}: {
  icon: typeof Shield;
  title: string;
  description: string;
  children: ReactNode;
}) {
  return (
    <div className="surface-card p-6">
      <div className="mb-6 flex items-start gap-4">
        <div className="rounded-[22px] border border-white/10 bg-white/5 p-3">
          <Icon className="h-5 w-5 text-sky-300" />
        </div>
        <div>
          <h2 className="text-xl font-semibold text-white">{title}</h2>
          <p className="mt-1 text-sm text-slate-400">{description}</p>
        </div>
      </div>
      {children}
    </div>
  );
}

function ToggleRow({
  label,
  checked,
  onChange,
}: {
  label: string;
  checked: boolean;
  onChange: (checked: boolean) => void;
}) {
  return (
    <label className="flex cursor-pointer items-center justify-between rounded-[22px] border border-white/10 bg-slate-950/40 px-4 py-4">
      <span className="text-sm text-slate-200">{label}</span>
      <button
        type="button"
        onClick={() => onChange(!checked)}
        className={`relative h-7 w-12 rounded-full transition-colors ${checked ? 'bg-sky-500' : 'bg-slate-700'}`}
      >
        <span
          className={`absolute top-1 h-5 w-5 rounded-full bg-white transition-transform ${checked ? 'translate-x-6' : 'translate-x-1'}`}
        />
      </button>
    </label>
  );
}
