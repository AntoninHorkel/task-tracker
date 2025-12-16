
import React, { useState, useContext } from 'react';
import { AuthContext } from '../context/AuthContext';
import { TaskContext } from '../context/TaskContext';
import { taskApi } from '../api/tasks';

export const TaskItem = ({ task }) => {
  const [isEditing, setIsEditing] = useState(false);
  const [title, setTitle] = useState(task.title);
  const [category, setCategory] = useState(task.category);
  const { state: authState } = useContext(AuthContext);
  const { dispatch } = useContext(TaskContext);

  const handleToggleComplete = async () => {
    try {
      const updated = await taskApi.updateTask(authState.token, task.id, {
        completed: !task.completed
      });
      dispatch({ type: 'UPDATE_TASK', payload: updated });
    } catch (err) {
      alert('Failed to update task: ' + err.message);
    }
  };

  const handleUpdate = async () => {
    try {
      const updated = await taskApi.updateTask(authState.token, task.id, {
        title,
        category
      });
      dispatch({ type: 'UPDATE_TASK', payload: updated });
      setIsEditing(false);
    } catch (err) {
      alert('Failed to update task: ' + err.message);
    }
  };

  const handleDelete = async () => {
    if (!confirm('Delete this task?')) return;
    try {
      await taskApi.deleteTask(authState.token, task.id);
      dispatch({ type: 'DELETE_TASK', payload: task.id });
    } catch (err) {
      alert('Failed to delete task: ' + err.message);
    }
  };

  if (isEditing) {
    return (
      <div className="bg-white p-4 rounded-lg shadow mb-3 border-2 border-blue-300">
        <input
          type="text"
          value={title}
          onChange={(e) => setTitle(e.target.value)}
          className="w-full px-3 py-2 border rounded-md mb-2"
          placeholder="Task title"
        />
        <input
          type="text"
          value={category}
          onChange={(e) => setCategory(e.target.value)}
          className="w-full px-3 py-2 border rounded-md mb-2"
          placeholder="Category"
        />
        <div className="flex gap-2">
          <button 
            onClick={handleUpdate} 
            className="px-4 py-2 bg-blue-600 text-white rounded hover:bg-blue-700"
          >
            Save
          </button>
          <button 
            onClick={() => setIsEditing(false)} 
            className="px-4 py-2 bg-gray-300 rounded hover:bg-gray-400"
          >
            Cancel
          </button>
        </div>
      </div>
    );
  }

  return (
    <div className="bg-white p-4 rounded-lg shadow mb-3 hover:shadow-md transition">
      <div className="flex items-start gap-3 mb-2">
        {}
        <input
          type="checkbox"
          checked={task.completed}
          onChange={handleToggleComplete}
          className="mt-1 w-5 h-5 cursor-pointer"
        />
        <div className="flex-1">
          <h3 className={`text-lg font-semibold ${task.completed ? 'line-through text-gray-500' : 'text-gray-800'}`}>
            {task.title}
          </h3>
          {}
          <span className="inline-block mt-1 px-3 py-1 bg-blue-100 text-blue-800 text-xs font-medium rounded-full">
            {task.category}
          </span>
        </div>
      </div>
      
      <div className="flex gap-2 mt-3">
        <button
          onClick={() => setIsEditing(true)}
          className="px-3 py-1 bg-blue-100 text-blue-700 rounded hover:bg-blue-200 text-sm"
        >
          Edit
        </button>
        <button
          onClick={handleDelete}
          className="px-3 py-1 bg-red-100 text-red-700 rounded hover:bg-red-200 text-sm"
        >
          Delete
        </button>
      </div>
    </div>
  );
};
