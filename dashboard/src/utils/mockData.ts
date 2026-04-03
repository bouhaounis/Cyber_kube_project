// Mock data for development and demo
import type { Alert } from '../stores/alertStore';
import type { Pod } from '../stores/clusterStore';
import type { Policy } from '../stores/policyStore';

export const mockAlerts: Alert[] = [
  {
    id: '1',
    kind: 'Container Escape',
    severity: 'high',
    message: 'Suspicious process detected attempting to escape container',
    createdAt: new Date(Date.now() - 1000 * 60 * 5).toISOString(),
  },
  {
    id: '2',
    kind: 'Network Scan',
    severity: 'medium',
    message: 'Multiple connection attempts detected from pod web-app-1',
    createdAt: new Date(Date.now() - 1000 * 60 * 15).toISOString(),
  },
  {
    id: '3',
    kind: 'Crypto Mining',
    severity: 'high',
    message: 'Unusual CPU usage pattern detected in pod worker-3',
    createdAt: new Date(Date.now() - 1000 * 60 * 30).toISOString(),
  },
];

export const mockPods: Pod[] = [
  { id: '1', name: 'web-app-1', namespace: 'default', status: 'Running', node: 'node-1' },
  { id: '2', name: 'api-server-1', namespace: 'default', status: 'Running', node: 'node-1' },
  { id: '3', name: 'worker-1', namespace: 'production', status: 'Running', node: 'node-2' },
  { id: '4', name: 'worker-2', namespace: 'production', status: 'Running', node: 'node-2' },
  { id: '5', name: 'worker-3', namespace: 'production', status: 'Running', node: 'node-3' },
  { id: '6', name: 'db-primary', namespace: 'database', status: 'Running', node: 'node-3' },
  { id: '7', name: 'cache-redis', namespace: 'cache', status: 'Running', node: 'node-1' },
  { id: '8', name: 'monitor-prometheus', namespace: 'monitoring', status: 'Running', node: 'node-2' },
];

export const mockPolicies: Policy[] = [
  {
    id: 'policy-1',
    name: 'Block Container Escape',
    description: 'Detect and block attempts to escape container boundaries',
    enabled: true,
    createdAt: new Date(Date.now() - 1000 * 60 * 60 * 24).toISOString(),
  },
  {
    id: 'policy-2',
    name: 'Network Isolation',
    description: 'Enforce network policies between namespaces',
    enabled: true,
    createdAt: new Date(Date.now() - 1000 * 60 * 60 * 48).toISOString(),
  },
  {
    id: 'policy-3',
    name: 'Privilege Escalation Prevention',
    description: 'Monitor and prevent privilege escalation attempts',
    enabled: true,
    createdAt: new Date(Date.now() - 1000 * 60 * 60 * 72).toISOString(),
  },
];
