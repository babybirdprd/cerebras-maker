import React, { useState, useEffect, useRef } from 'react';
import { Rewind, FastForward, Play, Pause, GitCommit, RotateCcw } from 'lucide-react';
import { getGitHistory, checkoutCommit, getSnapshots, HistoryEntry, Snapshot } from '../hooks/useTauri';

interface TimeSliderProps {
  onTimeTravel?: (commitHash: string) => void;
}

const TimeSlider: React.FC<TimeSliderProps> = ({ onTimeTravel }) => {
  const [isPlaying, setIsPlaying] = useState(false);
  const [value, setValue] = useState(0);
  const [history, setHistory] = useState<HistoryEntry[]>([]);
  const [snapshots, setSnapshots] = useState<Snapshot[]>([]);
  const [loading, setLoading] = useState(false);
  const [status, setStatus] = useState<string | null>(null);
  const timerRef = useRef<ReturnType<typeof setInterval> | null>(null);

  const max = Math.max(0, history.length - 1);
  const currentEntry = history[value];

  // Load git history on mount
  useEffect(() => {
    loadData();
  }, []);

  async function loadData() {
    try {
      const [historyData, snapshotData] = await Promise.all([
        getGitHistory(100),
        getSnapshots().catch(() => [] as Snapshot[])
      ]);
      setHistory(historyData);
      setSnapshots(snapshotData);
      setValue(0);
    } catch (e) {
      console.log('Failed to load git data:', e);
    }
  }

  useEffect(() => {
    if (isPlaying && max > 0) {
      timerRef.current = setInterval(() => {
        setValue((prev) => {
          if (prev >= max) {
            setIsPlaying(false);
            return max;
          }
          return prev + 1;
        });
      }, 500); // Slower for git history
    } else {
      if (timerRef.current) clearInterval(timerRef.current);
    }

    return () => {
      if (timerRef.current) clearInterval(timerRef.current);
    };
  }, [isPlaying, max]);

  const togglePlay = () => {
    if (value >= max && !isPlaying) {
      setValue(0);
    }
    setIsPlaying(!isPlaying);
  };

  const handleTimeTravel = async () => {
    if (!currentEntry) return;
    setLoading(true);
    setStatus(null);
    try {
      await checkoutCommit(currentEntry.hash);
      setStatus(`✅ Checked out to ${currentEntry.hash.slice(0, 7)}`);
      onTimeTravel?.(currentEntry.hash);
    } catch (e) {
      setStatus(`❌ Failed: ${e}`);
    }
    setLoading(false);
  };

  return (
    <div className="h-20 bg-zinc-950 border-t border-zinc-800 flex items-center px-4 md:px-6 gap-4 md:gap-6 z-20 shrink-0">
      <div className="flex items-center gap-1 md:gap-2">
         <button
           onClick={() => setValue(0)}
           className="p-1.5 md:p-2 hover:bg-zinc-800 rounded-full text-zinc-400 hover:text-white transition-colors"
           title="Go to latest"
         >
             <Rewind size={16} className="md:w-[18px] md:h-[18px]" />
         </button>
         <button
            className="p-2 md:p-3 bg-indigo-600 hover:bg-indigo-500 rounded-full text-white transition-colors shadow-lg shadow-indigo-500/20 active:scale-95"
            onClick={togglePlay}
            disabled={history.length === 0}
            title={isPlaying ? 'Pause' : 'Play through history'}
         >
             {isPlaying ? <Pause size={16} fill="currentColor" className="md:w-[18px] md:h-[18px]" /> : <Play size={16} fill="currentColor" className="md:w-[18px] md:h-[18px]" />}
         </button>
         <button
           onClick={() => setValue(max)}
           className="p-1.5 md:p-2 hover:bg-zinc-800 rounded-full text-zinc-400 hover:text-white transition-colors"
           title="Go to oldest"
         >
             <FastForward size={16} className="md:w-[18px] md:h-[18px]" />
         </button>
         <button
           onClick={handleTimeTravel}
           disabled={!currentEntry || loading}
           className="p-1.5 md:p-2 hover:bg-emerald-800 bg-emerald-900/50 rounded-full text-emerald-400 hover:text-white transition-colors disabled:opacity-50"
           title="Checkout to this commit"
         >
             <RotateCcw size={16} className="md:w-[18px] md:h-[18px]" />
         </button>
      </div>

      <div className="flex-1 flex flex-col justify-center select-none">
          <div className="flex justify-between text-xs font-mono mb-1.5">
              <span className="text-zinc-500 hidden sm:inline">
                {history.length > 0 ? history[history.length - 1]?.message?.slice(0, 20) || 'Initial' : 'No history'}
              </span>
              <span className="text-indigo-400 font-bold">
                {currentEntry ? (
                  <>SHA: {currentEntry.hash.slice(0, 7)} <span className="hidden sm:inline">({currentEntry.message?.slice(0, 30)})</span></>
                ) : (
                  'No commits'
                )}
              </span>
          </div>
          <div className="relative h-6 flex items-center group">
              <div className="absolute w-full h-1.5 bg-zinc-800 rounded-full overflow-hidden">
                  <div className="h-full bg-gradient-to-r from-indigo-900 to-indigo-500" style={{ width: `${max > 0 ? (value / max) * 100 : 0}%` }}></div>
              </div>

              {/* Snapshot markers */}
              {snapshots.map((snap) => {
                const snapIndex = history.findIndex(h => h.hash === snap.commit_hash);
                if (snapIndex < 0) return null;
                const pos = max > 0 ? (snapIndex / max) * 100 : 0;
                return (
                  <div
                    key={snap.id}
                    className="absolute w-2 h-2 bg-amber-500 rounded-full z-30 cursor-pointer"
                    style={{ left: `calc(${pos}% - 4px)` }}
                    title={`Snapshot: ${snap.message}`}
                    onClick={() => setValue(snapIndex)}
                  />
                );
              })}

              <div className="absolute w-full h-full flex justify-between pointer-events-none opacity-0 group-hover:opacity-100 transition-opacity duration-300 hidden sm:flex">
                  {Array.from({ length: Math.min(50, history.length) }).map((_, i) => (
                      <div key={i} className="w-[1px] h-2 bg-zinc-600 mt-2"></div>
                  ))}
              </div>

              <input
                type="range"
                min="0"
                max={max || 1}
                step="1"
                value={value}
                onChange={(e) => setValue(parseInt(e.target.value))}
                className="absolute w-full h-full opacity-0 cursor-ew-resize z-50 touch-none"
                disabled={history.length === 0}
              />

              <div
                className="absolute w-4 h-4 bg-white border-2 border-indigo-500 rounded-full shadow-lg pointer-events-none transition-all duration-75 z-40"
                style={{ left: `calc(${max > 0 ? (value / max) * 100 : 0}% - 8px)` }}
              ></div>
          </div>
          {status && (
            <div className={`text-xs mt-1 ${status.startsWith('✅') ? 'text-emerald-400' : 'text-red-400'}`}>
              {status}
            </div>
          )}
      </div>

      <div className="hidden sm:flex flex-col items-end min-w-[140px]">
          <div className="flex items-center gap-2 text-emerald-500">
              <GitCommit size={14} />
              <span className="font-mono font-bold text-sm">
                {snapshots.length > 0 ? `${snapshots.length} Snapshots` : 'Shadow Mode'}
              </span>
          </div>
          <span className="text-xs text-zinc-500">
            {history.length > 0 ? `Commit ${value + 1} / ${history.length}` : 'No history'}
          </span>
          <button
            onClick={loadData}
            className="text-xs text-zinc-500 hover:text-white mt-1"
          >
            🔄 Refresh
          </button>
      </div>
    </div>
  );
};

export default TimeSlider;

