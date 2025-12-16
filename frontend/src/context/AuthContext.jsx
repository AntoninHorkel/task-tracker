import { createContext, useReducer } from 'react';

export const AuthContext = createContext();

const authReducer = (state, action) => {
  switch (action.type) {
    case 'LOGIN':
      return { 
        token: action.payload.jwt, 
        username: action.payload.username, 
        isAuthenticated: true 
      };
    case 'LOGOUT':
      return { token: null, username: null, isAuthenticated: false };
    default:
      return state;
  }
};

export const AuthProvider = ({ children }) => {
  const [state, dispatch] = useReducer(authReducer, {
    token: localStorage.getItem('jwt'),
    username: localStorage.getItem('username'),
    isAuthenticated: !!localStorage.getItem('jwt')
  });

  return (
    <AuthContext.Provider value={{ state, dispatch }}>
      {children}
    </AuthContext.Provider>
  );
};