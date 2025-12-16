import React, { useContext } from 'react';
import { TaskContext } from '../context/TaskContext';
import { TaskItem } from './TaskItem';

export const TaskList = () => {
  const { state } = useContext(TaskContext);
  
  const filteredTasks = state.tasks.filter(task => {
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

  return (
    <div>
      <h2 className="text-2xl font-bold mb-4 text-gray-800">
        Tasks ({filteredTasks.length})
      </h2>
      {filteredTasks.length === 0 ? (
        <p className="text-gray-500 text-center py-8">No tasks found</p>
      ) : (
        filteredTasks.map(task => <TaskItem key={task.id} task={task} />)
      )}
    </div>
  );
};
