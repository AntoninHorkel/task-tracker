const API_BASE = import.meta.env.VITE_BACKEND_URL || 'http://localhost:6767';

export const taskApi = {
  async getAllTasks(jwt) {
    const res = await fetch(`${API_BASE}/task?jwt=${encodeURIComponent(jwt)}`, {
      method: 'GET',
      headers: { 'Content-Type': 'application/json' },
    });
    if (!res.ok) {
      const errorText = await res.text();
      throw new Error(errorText || 'Failed to fetch tasks');
    }
    return res.json();
  },

  async createTask(jwt, { category, title, text, completed = false, due = null }) {
    const res = await fetch(`${API_BASE}/task`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ jwt, category, title, text, completed, due })
    });
    if (!res.ok) {
      const errorText = await res.text();
      throw new Error(errorText || 'Failed to create task');
    }
    return true;
  },

  async updateTask(jwt, id, updates) {
    const res = await fetch(`${API_BASE}/task/${id}`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ jwt, ...updates })
    });
    if (!res.ok) {
      const errorText = await res.text();
      throw new Error(errorText || 'Failed to update task');
    }
    return true;
  },

  async deleteTask(jwt, id) {
    const res = await fetch(`${API_BASE}/task/${id}`, {
      method: 'DELETE',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ jwt })
    });
    if (!res.ok) {
      const errorText = await res.text();
      throw new Error(errorText || 'Failed to delete task');
    }
    return true;
  }
};
