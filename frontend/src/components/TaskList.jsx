import React, { useContext, useMemo } from 'react';
import { TaskContext } from '../context/TaskContext';
import { TaskItem } from './TaskItem';

export const TaskList = () => {
  const { state } = useContext(TaskContext);
  
  const filteredAndSortedTasks = useMemo(() => {
    let filtered = state.tasks.filter(task => {
      let passesStatusFilter = true;
      if (state.filter === 'completed') {
        passesStatusFilter = task.completed === true;
      } else if (state.filter === 'not_completed') {
        passesStatusFilter = task.completed === false;
      }

      let passesCategoryFilter = true;
      if (state.categoryFilter !== null) {
        passesCategoryFilter = task.category === state.categoryFilter;
      }

      return passesStatusFilter && passesCategoryFilter;
    });

    // Then, sort tasks
    if (state.sortBy === 'due_asc') {
      filtered = [...filtered].sort((a, b) => {
        // Tasks without due dates go to the end
        if (!a.due && !b.due) return 0;
        if (!a.due) return 1;
        if (!b.due) return -1;
        return a.due - b.due;
      });
    } else if (state.sortBy === 'due_desc') {
      filtered = [...filtered].sort((a, b) => {
        // Tasks without due dates go to the end
        if (!a.due && !b.due) return 0;
        if (!a.due) return 1;
        if (!b.due) return -1;
        return b.due - a.due;
      });
    }

    return filtered;
  }, [state.tasks, state.filter, state.categoryFilter, state.sortBy]);

  return (
    <div>
      <h2 className="text-2xl font-bold mb-4 text-gray-800">
        Tasks ({filteredAndSortedTasks.length})
      </h2>
      {filteredAndSortedTasks.length === 0 ? (
        <p className="text-gray-500 text-center py-8">No tasks found</p>
      ) : (
        filteredAndSortedTasks.map(task => <TaskItem key={task.id} task={task} />)
      )}
    </div>
  );
};
