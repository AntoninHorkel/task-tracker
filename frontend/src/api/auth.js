const API_BASE = 'http://localhost:6767';

export const authApi = {
  async register(username, password) {
    const res = await fetch(`${API_BASE}/auth/register`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ username, password })
    });
    if (!res.ok) throw new Error('Registration failed');
    return res.json();
  },

  async login(username, password) {
    const res = await fetch(`${API_BASE}/auth/login`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ username, password })
    });
    if (!res.ok) throw new Error('Login failed');
    return res.json();
  },

  async logout(jwt) {
    const res = await fetch(`${API_BASE}/auth/logout`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ jwt })
    });
    return res.ok;
  }
};