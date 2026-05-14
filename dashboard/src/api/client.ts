const TOKEN_STORAGE_KEY = 'ksoc_token';
const API_URL_STORAGE_KEY = 'ksoc_api_url';
const DEFAULT_API_URL = import.meta.env.VITE_API_URL || 'http://localhost:8081';

type LoginResponse = {
  token?: string;
  access_token?: string;
};

function getToken(): string | null {
  return localStorage.getItem(TOKEN_STORAGE_KEY);
}

function normalizeApiUrl(value: string): string {
  return value.trim().replace(/\/+$/, '');
}

function getApiUrl(): string {
  const stored = localStorage.getItem(API_URL_STORAGE_KEY);
  if (stored) {
    return normalizeApiUrl(stored);
  }
  return normalizeApiUrl(DEFAULT_API_URL);
}

function clearTokenAndRedirect(): void {
  localStorage.removeItem(TOKEN_STORAGE_KEY);
  if (window.location.pathname !== '/login') {
    window.location.assign('/login');
  }
}

async function request<T>(endpoint: string, init: RequestInit = {}): Promise<T> {
  const token = getToken();
  const headers = new Headers(init.headers);

  if (init.body && !headers.has('Content-Type')) {
    headers.set('Content-Type', 'application/json');
  }

  if (token) {
    headers.set('Authorization', `Bearer ${token}`);
  }

  const response = await fetch(`${getApiUrl()}${endpoint}`, {
    ...init,
    headers,
  });

  if (response.status === 401) {
    clearTokenAndRedirect();
    throw new Error('Unauthorized');
  }

  if (!response.ok) {
    const message = await extractErrorMessage(response);
    throw new Error(message || `HTTP error! status: ${response.status}`);
  }

  if (response.status === 204) {
    return undefined as T;
  }

  return response.json();
}

export const apiClient = {
  async get<T>(endpoint: string): Promise<T> {
    return request<T>(endpoint);
  },

  async post<T>(endpoint: string, data: unknown): Promise<T> {
    return request<T>(endpoint, {
      method: 'POST',
      body: JSON.stringify(data),
    });
  },

  async put<T>(endpoint: string, data: unknown): Promise<T> {
    return request<T>(endpoint, {
      method: 'PUT',
      body: JSON.stringify(data),
    });
  },

  async delete(endpoint: string): Promise<void> {
    await request<void>(endpoint, { method: 'DELETE' });
  },

  async login(username: string, password: string): Promise<string> {
    const response = await fetch(`${getApiUrl()}/api/v1/auth/login`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ username, password }),
    });

    if (response.status === 401) {
      clearTokenAndRedirect();
      throw new Error('Invalid username or password');
    }

    if (!response.ok) {
      const message = await extractErrorMessage(response);
      throw new Error(message || `HTTP error! status: ${response.status}`);
    }

    const data: LoginResponse = await response.json();
    const token = data.token ?? data.access_token;

    if (!token) {
      throw new Error('Login response did not include a token');
    }

    localStorage.setItem(TOKEN_STORAGE_KEY, token);
    return token;
  },

  getToken,
  getApiUrl,
  setApiUrl: (value: string) => localStorage.setItem(API_URL_STORAGE_KEY, normalizeApiUrl(value)),
  clearToken: () => localStorage.removeItem(TOKEN_STORAGE_KEY),
};

async function extractErrorMessage(response: Response): Promise<string | null> {
  try {
    const data = (await response.json()) as { error?: string; message?: string };
    return data.error ?? data.message ?? null;
  } catch {
    return null;
  }
}
