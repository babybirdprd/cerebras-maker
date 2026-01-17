// Cerebras-MAKER: The Time Machine Component
// PRD Section 6.2: Visual scrubber for gitoxide history

import { useEffect, useState } from 'react';
import { History, RefreshCw, Loader2, GitCommit, Undo2, Camera } from 'lucide-react';
import { useMakerStore } from '../store/makerStore';
import { getGitHistory, checkoutCommit, getSnapshots, rollbackToSnapshot, Snapshot } from '../hooks/useTauri';

export function TimeMachine() {
  const { gitHistory, currentCommit, setGitHistory, setCurrentCommit, workspacePath } = useMakerStore();
  const [loading, setLoading] = useState(false);
  const [sliderValue, setSliderValue] = useState(0);
  const [snapshots, setSnapshots] = useState<Snapshot[]>([]);
  const [timeTravelStatus, setTimeTravelStatus] = useState<string | null>(null);

  // Load git history and snapshots on mount
  useEffect(() => {
    if (workspacePath) {
      loadHistory();
      loadSnapshots();
    }
  }, [workspacePath]);

  async function loadHistory() {
    if (!workspacePath) return;
    setLoading(true);
    try {
      const history = await getGitHistory(workspacePath, 50);
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
    if (!currentCommit || !workspacePath) return;

    setLoading(true);
    setTimeTravelStatus(null);
    try {
      const result = await checkoutCommit(workspacePath, currentCommit);
      setTimeTravelStatus(`success:${result}`);
      // Refresh history after checkout
      await loadHistory();
    } catch (e) {
      setTimeTravelStatus(`error:Failed: ${e}`);
      console.error('Failed to time travel:', e);
    }
    setLoading(false);
  }

  async function handleRollbackToSnapshot(snapshotId: string) {
    setLoading(true);
    setTimeTravelStatus(null);
    try {
      const result = await rollbackToSnapshot(snapshotId);
      setTimeTravelStatus(`success:${result}`);
      // Refresh history and snapshots after rollback
      await loadHistory();
      await loadSnapshots();
    } catch (e) {
      setTimeTravelStatus(`error:Failed: ${e}`);
      console.error('Failed to rollback:', e);
    }
    setLoading(false);
  }

  const currentEntry = gitHistory[sliderValue];
  const isSuccess = timeTravelStatus?.startsWith('success:');
  const statusMessage = timeTravelStatus?.replace(/^(success|error):/, '');

  return (
    <div className="h-full flex flex-col bg-zinc-950 rounded-lg overflow-hidden">
      {/* Header */}
      <div className="p-4 border-b border-zinc-800 bg-zinc-900/50 flex justify-between items-center">
        <div className="flex items-center gap-2">
          <History className="text-indigo-500" size={18} />
          <h2 className="text-sm font-bold text-white">Time Machine</h2>
        </div>
        <button
          onClick={loadHistory}
          disabled={loading}
          className="flex items-center justify-center w-8 h-8 bg-zinc-800 hover:bg-zinc-700 text-zinc-300 rounded border border-zinc-700 transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
        >
          {loading ? (
            <Loader2 size={14} className="animate-spin" />
          ) : (
            <RefreshCw size={14} />
          )}
        </button>
      </div>

      {/* Timeline Slider Section */}
      <div className="p-4 lg:p-6 border-b border-zinc-800">
        {gitHistory.length === 0 ? (
          <div className="flex flex-col items-center justify-center py-8 text-zinc-500">
            <GitCommit size={32} className="mb-3 opacity-30" />
            <p className="text-sm italic">No commit history available</p>
          </div>
        ) : (
          <>
            {/* Slider */}
            <div className="mb-4">
              <input
                type="range"
                min={0}
                max={Math.max(0, gitHistory.length - 1)}
                value={sliderValue}
                onChange={handleSliderChange}
                className="w-full h-2 bg-zinc-800 rounded-lg appearance-none cursor-pointer accent-indigo-500
                  [&::-webkit-slider-thumb]:appearance-none [&::-webkit-slider-thumb]:w-4 [&::-webkit-slider-thumb]:h-4
                  [&::-webkit-slider-thumb]:bg-indigo-500 [&::-webkit-slider-thumb]:rounded-full [&::-webkit-slider-thumb]:border-2
                  [&::-webkit-slider-thumb]:border-white [&::-webkit-slider-thumb]:cursor-pointer [&::-webkit-slider-thumb]:shadow-lg"
              />
              <div className="flex justify-between mt-2 text-xs text-zinc-500">
                <span>Latest</span>
                <span>Oldest</span>
              </div>
            </div>

            {/* Current Commit Display */}
            <div className="bg-zinc-900 border border-zinc-800 rounded-lg p-3 mb-4">
              {currentEntry && (
                <>
                  <div className="font-mono text-sm text-indigo-400 mb-1">{currentEntry.hash}</div>
                  <div className="text-sm text-zinc-300">{currentEntry.message}</div>
                </>
              )}
            </div>

            {/* Time Travel Button */}
            <button
              onClick={handleTimeTravel}
              disabled={!currentCommit || loading}
              className="w-full flex items-center justify-center gap-2 px-4 py-2.5 bg-indigo-600 hover:bg-indigo-500 text-white font-semibold rounded-lg border border-indigo-500 transition-all disabled:opacity-50 disabled:cursor-not-allowed disabled:hover:bg-indigo-600"
            >
              {loading ? (
                <>
                  <Loader2 size={16} className="animate-spin" />
                  <span>Working...</span>
                </>
              ) : (
                <>
                  <Undo2 size={16} />
                  <span>Time Travel to This Commit</span>
                </>
              )}
            </button>

            {/* Status Message */}
            {timeTravelStatus && (
              <div className={`mt-3 p-3 rounded-lg text-sm text-center ${
                isSuccess
                  ? 'bg-emerald-900/20 border border-emerald-500/30 text-emerald-400'
                  : 'bg-rose-900/20 border border-rose-500/30 text-rose-400'
              }`}>
                {statusMessage}
              </div>
            )}
          </>
        )}
      </div>

      {/* Snapshots Section */}
      {snapshots.length > 0 && (
        <div className="p-4 border-b border-zinc-800">
          <div className="flex items-center gap-2 mb-3">
            <Camera size={14} className="text-amber-500" />
            <h3 className="text-xs uppercase font-bold tracking-wider text-zinc-400">Snapshots</h3>
          </div>
          <div className="flex flex-col gap-1">
            {snapshots.map((snapshot) => (
              <div
                key={snapshot.id}
                onClick={() => handleRollbackToSnapshot(snapshot.id)}
                className="flex items-center gap-3 p-2.5 bg-amber-500/5 hover:bg-amber-500/10 border border-amber-500/20 rounded-lg cursor-pointer transition-colors"
              >
                <span className="font-mono text-xs text-amber-400 min-w-[70px]">
                  {snapshot.id.slice(0, 8)}
                </span>
                <span className="flex-1 text-sm text-zinc-300 truncate">
                  {snapshot.message}
                </span>
                <span className="text-xs text-zinc-500">
                  {new Date(snapshot.timestamp_ms).toLocaleTimeString()}
                </span>
              </div>
            ))}
          </div>
        </div>
      )}

      {/* Recent Commits List */}
      <div className="flex-1 p-4 overflow-y-auto">
        <div className="flex items-center gap-2 mb-3">
          <GitCommit size={14} className="text-zinc-500" />
          <h3 className="text-xs uppercase font-bold tracking-wider text-zinc-400">Recent Commits</h3>
        </div>
        <div className="flex flex-col gap-1">
          {gitHistory.slice(0, 10).map((entry, idx) => (
            <div
              key={entry.hash}
              onClick={() => {
                setSliderValue(idx);
                setCurrentCommit(entry.hash);
              }}
              className={`flex items-center gap-3 p-2.5 rounded-lg cursor-pointer transition-all ${
                idx === sliderValue
                  ? 'bg-indigo-500/10 border border-indigo-500/30 border-l-2 border-l-indigo-500'
                  : 'bg-zinc-900/50 border border-transparent hover:bg-zinc-800/50'
              }`}
            >
              <span className={`font-mono text-xs min-w-[60px] ${
                idx === sliderValue ? 'text-indigo-400' : 'text-zinc-500'
              }`}>
                {entry.hash.slice(0, 7)}
              </span>
              <span className={`text-sm truncate ${
                idx === sliderValue ? 'text-white' : 'text-zinc-400'
              }`}>
                {entry.message}
              </span>
            </div>
          ))}
        </div>
      </div>
    </div>
  );
}

