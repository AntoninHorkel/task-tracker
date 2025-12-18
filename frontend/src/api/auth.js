const API_BASE = import.meta.env.VITE_BACKEND_URL || 'http://localhost:6767';

export const authApi = {
  async register(username, password) {
    const res = await fetch(`${API_BASE}/auth/register`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ username, password })
    });
    if (!res.ok) {
      const errorText = await res.text();
      throw new Error(errorText || 'Registration failed');
    }
    return res.json();
  },

  async login(username, password) {
    const res = await fetch(`${API_BASE}/auth/login`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ username, password })
    });
    if (!res.ok) {
      const errorText = await res.text();
      throw new Error(errorText || 'Login failed');
    }
    return res.json();
  },

  async refreshJWT(jwt) {
    const res = await fetch(`${API_BASE}/auth/login`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ jwt })
    });
    if (!res.ok) {
      const errorText = await res.text();
      throw new Error(errorText || 'JWT refresh failed');
    }
    return res.json();
  },

  async logout(jwt) {
    const res = await fetch(`${API_BASE}/auth/logout`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ jwt })
    });
    if (!res.ok) {
      const errorText = await res.text();
      throw new Error(errorText || 'Logout failed');
    }
    return true;
  }
};
