import React, { useState, useContext } from 'react';
import { AuthContext } from '../context/AuthContext';
import { TaskContext } from '../context/TaskContext';
import { taskApi } from '../api/tasks';

export const TaskForm = () => {
  const [category, setCategory] = useState('');
  const [title, setTitle] = useState('');
  const [text, setText] = useState('');
  const { state: authState } = useContext(AuthContext);
  const { dispatch } = useContext(TaskContext);

  const handleSubmit = async () => {
    if (!category.trim() || !text.trim()) {
      alert('Title and category are required');
      return;
    }

    try {
      const task = await taskApi.createTask(authState.token, { category: category, title: title, text: text });
      dispatch({ type: 'ADD_TASK', payload: task });
      setCategory('');
      setTitle('');
      setText('');
    } catch (err) {
      alert('Failed to create task: ' + err.message);
    }
  };

  return (
    <div className="bg-white p-6 rounded-lg shadow mb-6">
      <h2 className="text-xl font-bold mb-4">Create New Task</h2>
      <input
        type="text"
        placeholder="Category (e.g., work, personal) *"
        value={category}
        onChange={(e) => setCategory(e.target.value)}
        className="w-full px-3 py-2 border rounded-md mb-3 focus:outline-none focus:ring-2 focus:ring-blue-500"
      />
      {}
      <input
        type="text"
        placeholder="Task title *"
        value={title}
        onChange={(e) => setTitle(e.target.value)}
        className="w-full px-3 py-2 border rounded-md mb-3 focus:outline-none focus:ring-2 focus:ring-blue-500"
      />
      {}
      <input
        type="text"
        placeholder="Task text *"
        value={text}
        onChange={(e) => setText(e.target.value)}
        className="w-full px-3 py-2 border rounded-md mb-3 focus:outline-none focus:ring-2 focus:ring-blue-500"
      />
      <button
        onClick={handleSubmit}
        className="w-full px-6 py-2 bg-green-600 text-white rounded-md hover:bg-green-700 transition"
      >
        Create Task
      </button>
    </div>
  );
};
