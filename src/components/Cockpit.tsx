// Cerebras-MAKER: The Cockpit Component
// PRD Section 6.2: Real-time log of the Rhai execution loop

import { useEffect, useRef } from 'react';
import { useMakerStore, ExecutionEvent } from '../store/makerStore';
import { invoke } from '@tauri-apps/api/core';
import './Cockpit.css';

const EVENT_ICONS: Record<string, string> = {
  ScriptStart: '🚀',
  ScriptEnd: '✅',
  AtomSpawned: '⚛️',
  AtomCompleted: '✨',
  ConsensusStart: '🗳️',
  ConsensusVote: '📊',
  ConsensusEnd: '🏆',
  RedFlagDetected: '🚩',
  Snapshot: '📸',
  Rollback: '⏪',
  Error: '❌',
};

const EVENT_COLORS: Record<string, string> = {
  ScriptStart: '#4ade80',
  ScriptEnd: '#22c55e',
  AtomSpawned: '#60a5fa',
  AtomCompleted: '#3b82f6',
  ConsensusStart: '#a78bfa',
  ConsensusVote: '#8b5cf6',
  ConsensusEnd: '#7c3aed',
  RedFlagDetected: '#ef4444',
  Snapshot: '#fbbf24',
  Rollback: '#f97316',
  Error: '#dc2626',
};

function formatTimestamp(ms: number): string {
  const date = new Date(ms);
  const time = date.toLocaleTimeString('en-US', {
    hour12: false,
    hour: '2-digit',
    minute: '2-digit',
    second: '2-digit'
  });
  const millis = String(ms % 1000).padStart(3, '0');
  return `${time}.${millis}`;
}

function EventItem({ event }: { event: ExecutionEvent }) {
  const icon = EVENT_ICONS[event.event_type] || '📝';
  const color = EVENT_COLORS[event.event_type] || '#9ca3af';
  
  return (
    <div className="event-item" style={{ borderLeftColor: color }}>
      <span className="event-icon">{icon}</span>
      <span className="event-time">{formatTimestamp(event.timestamp_ms)}</span>
      <span className="event-type" style={{ color }}>{event.event_type}</span>
      <span className="event-message">{event.message}</span>
    </div>
  );
}

export function Cockpit() {
  const { executionLog, isExecuting, clearExecutionLog } = useMakerStore();
  const logEndRef = useRef<HTMLDivElement>(null);
  
  // Auto-scroll to bottom on new events
  useEffect(() => {
    logEndRef.current?.scrollIntoView({ behavior: 'smooth' });
  }, [executionLog]);
  
  // Poll for execution log updates
  useEffect(() => {
    const interval = setInterval(async () => {
      try {
        const log = await invoke<ExecutionEvent[]>('get_execution_log');
        if (log && log.length > executionLog.length) {
          log.slice(executionLog.length).forEach(event => {
            useMakerStore.getState().addExecutionEvent(event);
          });
        }
      } catch (e) {
        // Runtime not initialized yet
      }
    }, 500);
    
    return () => clearInterval(interval);
  }, [executionLog.length]);
  
  return (
    <div className="cockpit">
      <div className="cockpit-header">
        <h2>🎮 Cockpit</h2>
        <div className="cockpit-controls">
          <span className={`status-indicator ${isExecuting ? 'running' : 'idle'}`}>
            {isExecuting ? '● Running' : '○ Idle'}
          </span>
          <button onClick={clearExecutionLog} className="clear-btn">
            Clear Log
          </button>
        </div>
      </div>
      
      <div className="event-log">
        {executionLog.length === 0 ? (
          <div className="empty-log">
            No execution events yet. Initialize the runtime and run a script.
          </div>
        ) : (
          executionLog.map((event, index) => (
            <EventItem key={index} event={event} />
          ))
        )}
        <div ref={logEndRef} />
      </div>
    </div>
  );
}

