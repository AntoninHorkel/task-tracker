import React, { useState, useContext } from 'react';
import { AuthContext } from '../context/AuthContext';
import { authApi } from '../api/auth';

// Sanitization utilities
const sanitizeUsername = (username) => {
  // Trim whitespace and convert to lowercase
  return username.trim().toLowerCase();
};

const validateUsername = (username) => {
  const sanitized = sanitizeUsername(username);
  
  if (sanitized.length < 3) {
    return { valid: false, error: 'Username must be at least 3 characters long' };
  }
  
  if (sanitized.length > 30) {
    return { valid: false, error: 'Username must be less than 30 characters' };
  }
  
  // Only allow alphanumeric characters, underscores, and hyphens
  const usernameRegex = /^[a-z0-9_-]+$/;
  if (!usernameRegex.test(sanitized)) {
    return { valid: false, error: 'Username can only contain letters, numbers, underscores, and hyphens' };
  }
  
  return { valid: true, sanitized };
};

const sanitizePassword = (password) => {
  // For passwords, we typically don't trim or modify
  // Users might intentionally have spaces
  return password;
};

const validatePassword = (password, isRegistration = false) => {
  const sanitized = sanitizePassword(password);
  
  if (sanitized.length < 8) {
    return { valid: false, error: 'Password must be at least 8 characters long' };
  }
  
  if (sanitized.length > 128) {
    return { valid: false, error: 'Password must be less than 128 characters' };
  }
  
  // Additional validation for registration
  if (isRegistration) {
    const hasUpperCase = /[A-Z]/.test(sanitized);
    const hasLowerCase = /[a-z]/.test(sanitized);
    const hasNumber = /[0-9]/.test(sanitized);
    const hasSpecialChar = /[!@#$%^&*()_+\-=\[\]{};':"\\|,.<>\/?]/.test(sanitized);
    
    if (!hasUpperCase || !hasLowerCase || !hasNumber || !hasSpecialChar) {
      return { 
        valid: false, 
        error: 'Password must contain uppercase, lowercase, number, and special character' 
      };
    }
  }
  
  return { valid: true, sanitized };
};

export const AuthPage = () => {
  const [isLogin, setIsLogin] = useState(true);
  const [username, setUsername] = useState('');
  const [password, setPassword] = useState('');
  const [error, setError] = useState('');
  const [loading, setLoading] = useState(false);
  const { dispatch } = useContext(AuthContext);

  const handleUsernameChange = (e) => {
    const value = e.target.value;
    // Filter out invalid characters as user types
    const filtered = value.replace(/[^a-zA-Z0-9_-]/g, '');
    setUsername(filtered);
    setError(''); // Clear error when user types
  };

  const handlePasswordChange = (e) => {
    setPassword(e.target.value);
    setError(''); // Clear error when user types
  };

  const handleSubmit = async () => {
    setError('');
    
    // Validate username
    const usernameValidation = validateUsername(username);
    if (!usernameValidation.valid) {
      setError(usernameValidation.error);
      return;
    }
    
    // Validate password
    const passwordValidation = validatePassword(password, !isLogin);
    if (!passwordValidation.valid) {
      setError(passwordValidation.error);
      return;
    }

    setLoading(true);

    try {
      const sanitizedUsername = usernameValidation.sanitized;
      const sanitizedPassword = passwordValidation.sanitized;
      
      if (isLogin) {
        const data = await authApi.login(sanitizedUsername, sanitizedPassword);
        dispatch({ type: 'LOGIN', payload: data });
        localStorage.setItem('jwt', data.jwt);
        localStorage.setItem('username', data.username);
      } else {
        const data = await authApi.register(sanitizedUsername, sanitizedPassword);
        dispatch({ type: 'LOGIN', payload: { jwt: data.jwt, username: sanitizedUsername } });
        localStorage.setItem('jwt', data.jwt);
        localStorage.setItem('username', sanitizedUsername);
      }
    } catch (err) {
      setError(err.message || 'An error occurred. Please try again.');
    } finally {
      setLoading(false);
    }
  };

  const handleKeyPress = (e) => {
    if (e.key === 'Enter') {
      handleSubmit();
    }
  };

  return (
    <div className="min-h-screen bg-gray-100 flex items-center justify-center p-4">
      <div className="bg-white rounded-lg shadow-lg p-8 w-full max-w-md">
        <h1 className="text-3xl font-bold text-center mb-6 text-gray-800">
          {isLogin ? 'Login' : 'Register'}
        </h1>
        
        <div className="space-y-4">
          <div>
            <label className="block text-sm font-medium text-gray-700 mb-1">
              Username
            </label>
            <input
              type="text"
              value={username}
              onChange={handleUsernameChange}
              onKeyPress={handleKeyPress}
              className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
              placeholder="username123"
              maxLength={30}
            />
            <p className="text-xs text-gray-500 mt-1">
              3-30 characters, letters, numbers, _ and - only
            </p>
          </div>
          
          <div>
            <label className="block text-sm font-medium text-gray-700 mb-1">
              Password
            </label>
            <input
              type="password"
              value={password}
              onChange={handlePasswordChange}
              onKeyPress={handleKeyPress}
              className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
              placeholder="••••••••"
              maxLength={128}
            />
            <p className="text-xs text-gray-500 mt-1">
              {isLogin 
                ? 'At least 8 characters' 
                : 'At least 8 characters with uppercase, lowercase, number, and special character'}
            </p>
          </div>

          {error && (
            <div className="text-red-600 text-sm bg-red-50 p-3 rounded border border-red-200">
              {error}
            </div>
          )}

          <button
            onClick={handleSubmit}
            disabled={loading || !username || !password}
            className="w-full bg-blue-600 text-white py-2 rounded-md hover:bg-blue-700 disabled:bg-gray-400 disabled:cursor-not-allowed transition"
          >
            {loading ? 'Loading...' : (isLogin ? 'Login' : 'Register')}
          </button>
        </div>

        <button
          onClick={() => {
            setIsLogin(!isLogin);
            setError('');
          }}
          className="w-full mt-4 text-blue-600 hover:underline text-sm"
        >
          {isLogin ? "Don't have an account? Register" : 'Already have an account? Login'}
        </button>
      </div>
    </div>
  );
};
