import React, { useState, useContext } from 'react';
import { AuthContext } from '../context/AuthContext';
import { TaskContext } from '../context/TaskContext';
import { taskApi } from '../api/tasks';

export const TaskItem = ({ task }) => {
  const [isEditing, setIsEditing] = useState(false);
  const [category, setCategory] = useState(task.category);
  const [title, setTitle] = useState(task.title);
  const [text, setText] = useState(task.text);
  const [dueDateTime, setDueDateTime] = useState(() => {
    if (task.due) {
      const date = new Date(task.due * 1000);
      return date.toISOString().slice(0, 16);
    }
    return '';
  });
  const { state: authState } = useContext(AuthContext);
  const { dispatch } = useContext(TaskContext);

  const handleToggleComplete = async () => {
    try {
      const updated = await taskApi.updateTask(authState.token, task.id, {
        ...task,
        completed: !task.completed
      });
      dispatch({ type: 'UPDATE_TASK', payload: updated });
    } catch (err) {
      alert('Failed to update task: ' + err.message);
    }
  };

  const handleUpdate = async () => {
    try {
      const updateData = {
        ...task,
        category: category,
        title: title,
        text: text,
      };

      if (dueDateTime) {
        updateData.due = Math.floor(new Date(dueDateTime).getTime() / 1000);
      } else {
        updateData.due = null;
      }

      const updated = await taskApi.updateTask(authState.token, task.id, updateData);
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

  const formatDueDate = (timestamp) => {
    if (!timestamp) return null;
    const date = new Date(timestamp * 1000);
    const now = new Date();
    const diffMs = date - now;
    const diffHours = Math.floor(diffMs / (1000 * 60 * 60));
    const diffDays = Math.floor(diffMs / (1000 * 60 * 60 * 24));
    
    const dateStr = date.toLocaleDateString('en-US', { 
      month: 'short', 
      day: 'numeric',
      year: date.getFullYear() !== now.getFullYear() ? 'numeric' : undefined
    });
    const timeStr = date.toLocaleTimeString('en-US', { 
      hour: 'numeric', 
      minute: '2-digit'
    });

    let color = 'text-gray-600';
    let bgColor = 'bg-gray-100';
    
    if (diffMs < 0) {
      color = 'text-red-700';
      bgColor = 'bg-red-100';
    } else if (diffHours < 24) {
      color = 'text-orange-700';
      bgColor = 'bg-orange-100';
    } else if (diffDays < 3) {
      color = 'text-yellow-700';
      bgColor = 'bg-yellow-100';
    }

    return { dateStr, timeStr, color, bgColor, isPast: diffMs < 0 };
  };

  const dueInfo = formatDueDate(task.due);

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
        <input
          type="text"
          value={text}
          onChange={(e) => setText(e.target.value)}
          className="w-full px-3 py-2 border rounded-md mb-2"
          placeholder="Task text"
        />
        <div className="mb-2">
          <label className="block text-sm font-medium text-gray-700 mb-1">
            Due Date & Time
          </label>
          <input
            type="datetime-local"
            value={dueDateTime}
            onChange={(e) => setDueDateTime(e.target.value)}
            className="w-full px-3 py-2 border rounded-md"
          />
        </div>
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
          <div className="flex gap-2 mt-1 flex-wrap">
            <span className="inline-block px-3 py-1 bg-blue-100 text-blue-800 text-xs font-medium rounded-full">
              {task.category}
            </span>
            {dueInfo && (
              <span className={`inline-block px-3 py-1 ${dueInfo.bgColor} ${dueInfo.color} text-xs font-medium rounded-full`}>
                {dueInfo.isPast ? '⚠ Overdue: ' : '📅 '}{dueInfo.dateStr} at {dueInfo.timeStr}
              </span>
            )}
          </div>
          <p className="text-sm font-semibold text-gray-800 mt-2">
            {task.text}
          </p>
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
