import React, { useState, useEffect, useRef } from 'react';
import { Rewind, FastForward, Play, Pause, GitCommit } from 'lucide-react';

const TimeSlider: React.FC = () => {
  const [isPlaying, setIsPlaying] = useState(false);
  const [value, setValue] = useState(483);
  const max = 500;
  const timerRef = useRef<ReturnType<typeof setInterval> | null>(null);

  useEffect(() => {
    if (isPlaying) {
      timerRef.current = setInterval(() => {
        setValue((prev) => {
          if (prev >= max) {
            setIsPlaying(false);
            return max;
          }
          return prev + 1;
        });
      }, 50);
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

  return (
    <div className="h-16 bg-zinc-950 border-t border-zinc-800 flex items-center px-4 md:px-6 gap-4 md:gap-6 z-20 shrink-0">
      <div className="flex items-center gap-1 md:gap-2">
         <button 
           onClick={() => setValue(0)}
           className="p-1.5 md:p-2 hover:bg-zinc-800 rounded-full text-zinc-400 hover:text-white transition-colors"
         >
             <Rewind size={16} className="md:w-[18px] md:h-[18px]" />
         </button>
         <button 
            className="p-2 md:p-3 bg-indigo-600 hover:bg-indigo-500 rounded-full text-white transition-colors shadow-lg shadow-indigo-500/20 active:scale-95"
            onClick={togglePlay}
         >
             {isPlaying ? <Pause size={16} fill="currentColor" className="md:w-[18px] md:h-[18px]" /> : <Play size={16} fill="currentColor" className="md:w-[18px] md:h-[18px]" />}
         </button>
         <button 
           onClick={() => setValue(max)}
           className="p-1.5 md:p-2 hover:bg-zinc-800 rounded-full text-zinc-400 hover:text-white transition-colors"
         >
             <FastForward size={16} className="md:w-[18px] md:h-[18px]" />
         </button>
      </div>

      <div className="flex-1 flex flex-col justify-center select-none">
          <div className="flex justify-between text-xs font-mono mb-1.5">
              <span className="text-zinc-500 hidden sm:inline">Initial Commit</span>
              <span className="text-indigo-400 font-bold">SHA: 7a9f2b <span className="hidden sm:inline">(Current)</span></span>
          </div>
          <div className="relative h-6 flex items-center group">
              <div className="absolute w-full h-1.5 bg-zinc-800 rounded-full overflow-hidden">
                  <div className="h-full bg-gradient-to-r from-indigo-900 to-indigo-500" style={{ width: `${(value / max) * 100}%` }}></div>
              </div>
              
              <div className="absolute w-full h-full flex justify-between pointer-events-none opacity-0 group-hover:opacity-100 transition-opacity duration-300 hidden sm:flex">
                  {Array.from({ length: 50 }).map((_, i) => (
                      <div key={i} className="w-[1px] h-2 bg-zinc-600 mt-2"></div>
                  ))}
              </div>

              <input 
                type="range" 
                min="0" 
                max={max} 
                step="1"
                value={value} 
                onChange={(e) => setValue(parseInt(e.target.value))}
                className="absolute w-full h-full opacity-0 cursor-ew-resize z-50 touch-none"
              />
              
              <div 
                className="absolute w-4 h-4 bg-white border-2 border-indigo-500 rounded-full shadow-lg pointer-events-none transition-all duration-75 z-40"
                style={{ left: `calc(${(value / max) * 100}% - 8px)` }}
              ></div>
          </div>
      </div>

      <div className="hidden sm:flex flex-col items-end min-w-[120px]">
          <div className="flex items-center gap-2 text-emerald-500">
              <GitCommit size={14} />
              <span className="font-mono font-bold text-sm">Shadow Mode</span>
          </div>
          <span className="text-xs text-zinc-500">Step {value} / {max}</span>
      </div>
    </div>
  );
};

export default TimeSlider;

