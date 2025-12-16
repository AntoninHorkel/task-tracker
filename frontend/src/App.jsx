import React from 'react';
import { AuthProvider, AuthContext } from './context/AuthContext';
import { TaskProvider } from './context/TaskContext';
import { AuthPage } from './pages/AuthPage';
import { Dashboard } from './pages/Dashboard';

const AppContent = () => {
  const { state } = React.useContext(AuthContext);
  return state.isAuthenticated ? <Dashboard /> : <AuthPage />;
};

const App = () => {
  return (
    <AuthProvider>
      <TaskProvider>
        <AppContent />
      </TaskProvider>
    </AuthProvider>
  );
};

export default App;