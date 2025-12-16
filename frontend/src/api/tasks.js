export const taskApi = {
  async getTasks(jwt) {
    const res = await fetch(`${API_BASE}/task`, {
      method: 'GET',
      headers: { 
        'Content-Type': 'application/json',
        'Authorization': `Bearer ${jwt}`
      }
    });
    if (!res.ok) throw new Error('Failed to fetch tasks');
    return res.json();
  },

  async createTask(jwt, title, category) {
    const res = await fetch(`${API_BASE}/task`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ jwt, title, category })
    });
    if (!res.ok) throw new Error('Failed to create task');
    return res.json();
  },

  async updateTask(jwt, id, updates) {
    const res = await fetch(`${API_BASE}/task/${id}`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ jwt, ...updates })
    });
    if (!res.ok) throw new Error('Failed to update task');
    return res.json();
  },
  async deleteTask(jwt, id) {
    const res = await fetch(`${API_BASE}/task/${id}`, {
      method: 'DELETE',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ jwt })
    });
    return res.ok;
  }
};
