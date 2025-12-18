import React, { useContext, useMemo } from 'react';
import { TaskContext } from '../context/TaskContext';

export const Filters = () => {
  const { state, dispatch } = useContext(TaskContext);

  const categories = useMemo(() => {
    const cats = new Set(state.tasks.map(t => t.category));
    return Array.from(cats).sort();
  }, [state.tasks]);

  return (
    <div className="mb-6 space-y-3">
      <div>
        <label className="block text-sm font-medium text-gray-700 mb-2">
          Status
        </label>
        <div className="flex gap-2">
          {['all', 'completed', 'not_completed'].map(filter => (
            <button
              key={filter}
              onClick={() => dispatch({ type: 'SET_FILTER', payload: filter })}
              className={`px-4 py-2 rounded-md transition text-sm ${
                state.filter === filter
                  ? 'bg-blue-600 text-white'
                  : 'bg-gray-200 text-gray-700 hover:bg-gray-300'
              }`}
            >
              {filter === 'not_completed' ? 'Not Completed' : filter.charAt(0).toUpperCase() + filter.slice(1)}
            </button>
          ))}
        </div>
      </div>

      {categories.length > 0 && (
        <div>
          <label className="block text-sm font-medium text-gray-700 mb-2">
            Category
          </label>
          <div className="flex gap-2 flex-wrap">
            <button
              onClick={() => dispatch({ type: 'SET_CATEGORY_FILTER', payload: null })}
              className={`px-4 py-2 rounded-md transition text-sm ${
                state.categoryFilter === null
                  ? 'bg-purple-600 text-white'
                  : 'bg-gray-200 text-gray-700 hover:bg-gray-300'
              }`}
            >
              All Categories
            </button>
            {categories.map(cat => (
              <button
                key={cat}
                onClick={() => dispatch({ type: 'SET_CATEGORY_FILTER', payload: cat })}
                className={`px-4 py-2 rounded-md transition text-sm ${
                  state.categoryFilter === cat
                    ? 'bg-purple-600 text-white'
                    : 'bg-gray-200 text-gray-700 hover:bg-gray-300'
                }`}
              >
                {cat}
              </button>
            ))}
          </div>
        </div>
      )}

      <div>
        <label className="block text-sm font-medium text-gray-700 mb-2">
          Sort By
        </label>
        <div className="flex gap-2 flex-wrap">
          {[
            { value: 'none', label: 'Default' },
            { value: 'due_asc', label: 'Due Date (Earliest First)' },
            { value: 'due_desc', label: 'Due Date (Latest First)' }
          ].map(option => (
            <button
              key={option.value}
              onClick={() => dispatch({ type: 'SET_SORT', payload: option.value })}
              className={`px-4 py-2 rounded-md transition text-sm ${
                state.sortBy === option.value
                  ? 'bg-green-600 text-white'
                  : 'bg-gray-200 text-gray-700 hover:bg-gray-300'
              }`}
            >
              {option.label}
            </button>
          ))}
        </div>
      </div>
    </div>
  );
};
