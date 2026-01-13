// Cerebras-MAKER: The Time Machine Component
// PRD Section 6.2: Visual scrubber for gitoxide history

import { useEffect, useState } from 'react';
import { useMakerStore } from '../store/makerStore';
import { getGitHistory, checkoutCommit, getSnapshots, rollbackToSnapshot, Snapshot } from '../hooks/useTauri';
import './TimeMachine.css';

export function TimeMachine() {
  const { gitHistory, currentCommit, setGitHistory, setCurrentCommit } = useMakerStore();
  const [loading, setLoading] = useState(false);
  const [sliderValue, setSliderValue] = useState(0);
  const [snapshots, setSnapshots] = useState<Snapshot[]>([]);
  const [timeTravelStatus, setTimeTravelStatus] = useState<string | null>(null);

  // Load git history and snapshots on mount
  useEffect(() => {
    loadHistory();
    loadSnapshots();
  }, []);

  async function loadHistory() {
    setLoading(true);
    try {
      const history = await getGitHistory(50);
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

  async function loadSnapshots() {
    try {
      const snaps = await getSnapshots();
      setSnapshots(snaps);
    } catch (e) {
      // Snapshots not available yet
      console.log('Snapshots not loaded:', e);
    }
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

    setLoading(true);
    setTimeTravelStatus(null);
    try {
      const result = await checkoutCommit(currentCommit);
      setTimeTravelStatus(`✅ ${result}`);
      // Refresh history after checkout
      await loadHistory();
    } catch (e) {
      setTimeTravelStatus(`❌ Failed: ${e}`);
      console.error('Failed to time travel:', e);
    }
    setLoading(false);
  }

  async function handleRollbackToSnapshot(snapshotId: string) {
    setLoading(true);
    setTimeTravelStatus(null);
    try {
      const result = await rollbackToSnapshot(snapshotId);
      setTimeTravelStatus(`✅ ${result}`);
      // Refresh history and snapshots after rollback
      await loadHistory();
      await loadSnapshots();
    } catch (e) {
      setTimeTravelStatus(`❌ Failed: ${e}`);
      console.error('Failed to rollback:', e);
    }
    setLoading(false);
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
              disabled={!currentCommit || loading}
            >
              {loading ? '⏳ Working...' : '🚀 Time Travel to This Commit'}
            </button>

            {timeTravelStatus && (
              <div className={`status-message ${timeTravelStatus.startsWith('✅') ? 'success' : 'error'}`}>
                {timeTravelStatus}
              </div>
            )}
          </>
        )}
      </div>

      {/* Snapshots Section */}
      {snapshots.length > 0 && (
        <div className="snapshots-section">
          <h3>📸 Snapshots</h3>
          <div className="snapshots">
            {snapshots.map((snapshot) => (
              <div
                key={snapshot.id}
                className="snapshot-item"
                onClick={() => handleRollbackToSnapshot(snapshot.id)}
              >
                <span className="snapshot-id">{snapshot.id.slice(0, 8)}</span>
                <span className="snapshot-message">{snapshot.message}</span>
                <span className="snapshot-time">
                  {new Date(snapshot.timestamp_ms).toLocaleTimeString()}
                </span>
              </div>
            ))}
          </div>
        </div>
      )}

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

