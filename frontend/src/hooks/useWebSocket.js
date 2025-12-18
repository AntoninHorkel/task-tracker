import { useEffect, useRef, useContext } from 'react';
import { TaskContext } from '../context/TaskContext';

export const useWebSocket = (token) => {
  const wsRef = useRef(null);
  const { dispatch } = useContext(TaskContext);

  useEffect(() => {
    if (!token) return;

    const ws = new WebSocket(`ws://localhost:6767/websocket?jwt=${token}`);
    wsRef.current = ws;

    ws.onopen = () => {
      // console.log('WebSocket connected');
    };
    
    ws.onmessage = (event) => {
      try {
        const data = JSON.parse(event.data);
        // console.log('WebSocket message:', data);
        
        switch (data.type) {
          case 'task_created':
            dispatch({ type: 'ADD_TASK', payload: data.task });
            break;
          
          case 'task_updated':
            dispatch({ type: 'UPDATE_TASK', payload: data.task });
            break;
          
          case 'task_deleted':
            dispatch({ type: 'DELETE_TASK', payload: data.task_id });
            break;
          
          default:
            console.warn('Unknown WebSocket message type:', data.type);
        }
      } catch (err) {
        console.error('WebSocket parse error:', err);
      }
    };

    ws.onerror = (err) => {
      console.error('WebSocket error:', err);
    };

    // ws.onclose = () => {
    //   console.log('WebSocket disconnected');
    // };

    return () => {
      if (ws.readyState === WebSocket.OPEN) {
        ws.close();
      }
    };
  }, [token, dispatch]);

  return wsRef;
};
