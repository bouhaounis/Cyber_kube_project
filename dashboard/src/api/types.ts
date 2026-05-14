export interface Policy {
  id: string;
  name: string;
  description: string;
  createdAt: string;
}

export interface Alert {
  id: string;
  kind: string;
  severity: string;
  message: string;
  createdAt: string;
}

export interface HealthStatus {
  status: string;
}

export interface AppSettings {
  email_notifications: boolean;
  notification_email: string;
  auto_remediation: boolean;
  log_retention_days: number;
}
