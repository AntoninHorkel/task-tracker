import React, { useContext, useEffect, useState } from 'react';
import { AuthContext } from '../context/AuthContext';
import { TaskContext } from '../context/TaskContext';
import { authApi } from '../api/auth';
import { taskApi } from '../api/tasks';
import { useWebSocket } from '../hooks/useWebSocket';
import { TaskForm } from '../components/TaskForm';
import { Filters } from '../components/Filters';
import { TaskList } from '../components/TaskList';

export const Dashboard = () => {
  const { state: authState, dispatch: authDispatch } = useContext(AuthContext);
  const { dispatch: taskDispatch } = useContext(TaskContext);
  const [loading, setLoading] = useState(true);

  useWebSocket(authState.token);

  useEffect(() => {
    const loadTasks = async () => {
      try {
        const tasks = await taskApi.getAllTasks(authState.token);
        taskDispatch({ type: 'SET_TASKS', payload: tasks });
      } catch (err) {
        console.error('Failed to load tasks:', err);
      } finally {
        setLoading(false);
      }
    };
    loadTasks();
  }, [authState.token, taskDispatch]);

  const handleLogout = async () => {
    await authApi.logout(authState.token);
    authDispatch({ type: 'LOGOUT' });
    localStorage.removeItem('jwt');
    localStorage.removeItem('username');
  };

  if (loading) {
    return (
      <div className="min-h-screen bg-gray-100 flex items-center justify-center">
        <div className="text-xl text-gray-600">Loading tasks...</div>
      </div>
    );
  }

  return (
    <div className="min-h-screen bg-gray-100 p-4">
      <div className="max-w-4xl mx-auto">
        <div className="flex justify-between items-center mb-6">
          <div>
            <h1 className="text-3xl font-bold text-gray-800">Task Tracker</h1>
            {authState.username && (
              <p className="text-sm text-gray-600">Welcome, {authState.username}</p>
            )}
          </div>
          <button
            onClick={handleLogout}
            className="px-4 py-2 bg-red-600 text-white rounded-md hover:bg-red-700 transition"
          >
            Logout
          </button>
        </div>

        <TaskForm />
        <Filters />
        <TaskList />
      </div>
    </div>
  );
};
