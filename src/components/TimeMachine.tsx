// Cerebras-MAKER: The Time Machine Component
// PRD Section 6.2: Visual scrubber for gitoxide history

import { useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { useMakerStore, HistoryEntry } from '../store/makerStore';
import './TimeMachine.css';

export function TimeMachine() {
  const { gitHistory, currentCommit, setGitHistory, setCurrentCommit } = useMakerStore();
  const [loading, setLoading] = useState(false);
  const [sliderValue, setSliderValue] = useState(0);
  
  // Load git history on mount
  useEffect(() => {
    loadHistory();
  }, []);
  
  async function loadHistory() {
    setLoading(true);
    try {
      const history = await invoke<HistoryEntry[]>('get_git_history', { limit: 50 });
      setGitHistory(history);
      if (history.length > 0) {
        setCurrentCommit(history[0].hash);
        setSliderValue(0);
      }
    } catch (e) {
      console.error('Failed to load git history:', e);
    }
    setLoading(false);
  }
  
  function handleSliderChange(e: React.ChangeEvent<HTMLInputElement>) {
    const value = parseInt(e.target.value, 10);
    setSliderValue(value);
    if (gitHistory[value]) {
      setCurrentCommit(gitHistory[value].hash);
    }
  }
  
  async function handleTimeTravel() {
    if (!currentCommit) return;
    
    try {
      // This would trigger a git checkout to the selected commit
      // For now, just update the UI state
      console.log('Time travel to:', currentCommit);
    } catch (e) {
      console.error('Failed to time travel:', e);
    }
  }
  
  const currentEntry = gitHistory[sliderValue];
  
  return (
    <div className="time-machine">
      <div className="time-machine-header">
        <h2>⏰ Time Machine</h2>
        <button onClick={loadHistory} disabled={loading} className="refresh-btn">
          {loading ? '...' : '🔄'}
        </button>
      </div>
      
      <div className="timeline-container">
        {gitHistory.length === 0 ? (
          <div className="empty-history">No commit history available</div>
        ) : (
          <>
            <div className="timeline-slider">
              <input
                type="range"
                min={0}
                max={Math.max(0, gitHistory.length - 1)}
                value={sliderValue}
                onChange={handleSliderChange}
                className="slider"
              />
              <div className="slider-labels">
                <span>Latest</span>
                <span>Oldest</span>
              </div>
            </div>
            
            <div className="current-commit">
              {currentEntry && (
                <>
                  <div className="commit-hash">{currentEntry.hash}</div>
                  <div className="commit-message">{currentEntry.message}</div>
                </>
              )}
            </div>
            
            <button 
              onClick={handleTimeTravel} 
              className="time-travel-btn"
              disabled={!currentCommit}
            >
              🚀 Time Travel to This Commit
            </button>
          </>
        )}
      </div>
      
      <div className="commit-list">
        <h3>Recent Commits</h3>
        <div className="commits">
          {gitHistory.slice(0, 10).map((entry, idx) => (
            <div 
              key={entry.hash}
              className={`commit-item ${idx === sliderValue ? 'selected' : ''}`}
              onClick={() => {
                setSliderValue(idx);
                setCurrentCommit(entry.hash);
              }}
            >
              <span className="commit-hash-small">{entry.hash.slice(0, 7)}</span>
              <span className="commit-msg-small">{entry.message}</span>
            </div>
          ))}
        </div>
      </div>
    </div>
  );
}

