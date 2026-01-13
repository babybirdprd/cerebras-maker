import { useEffect, useState } from 'react';
import { Cpu, AlertTriangle, CheckCircle, XCircle, GitCommit } from 'lucide-react';
import { CANDIDATES } from '../constants';
import { getExecutionLog, checkRedFlags } from '../hooks/useTauri';

interface ExecutionStats {
  agentCount: number;
  tokensPerSecond: number;
  redFlagCount: number;
  shadowCommits: number;
}

interface ExecutionPanelProps {
  isActive?: boolean;
}

const ExecutionPanel: React.FC<ExecutionPanelProps> = ({ isActive = false }) => {
  const [stats, setStats] = useState<ExecutionStats>({
    agentCount: 50,
    tokensPerSecond: 24000,
    redFlagCount: 0,
    shadowCommits: 0,
  });
  const [candidates] = useState(CANDIDATES);

  useEffect(() => {
    if (!isActive) return;

    const fetchData = async () => {
      try {
        // Fetch execution log
        const log = await getExecutionLog();
        if (Array.isArray(log)) {
          setStats(prev => ({ ...prev, shadowCommits: log.length }));
        }

        // Fetch red flags
        const redFlags = await checkRedFlags(0);
        setStats(prev => ({
          ...prev,
          redFlagCount: redFlags.cycles_detected?.length || 0
        }));
      } catch (error) {
        // Use mock data if Tauri not available
        console.log('Using mock execution data');
      }
    };

    fetchData();
    const interval = setInterval(fetchData, 5000); // Poll every 5 seconds
    return () => clearInterval(interval);
  }, [isActive]);
  return (
    <div className="h-full flex flex-col gap-6 p-4 lg:p-6 overflow-y-auto">
      {/* Header Stats */}
      <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
        <div className="bg-zinc-900 border border-zinc-800 p-4 rounded-lg relative overflow-hidden group">
            <div className="absolute inset-0 bg-gradient-to-r from-indigo-500/10 to-transparent opacity-0 group-hover:opacity-100 transition-opacity"></div>
            <div className="flex justify-between items-start">
                <div>
                    <h3 className="text-zinc-400 text-xs uppercase font-bold tracking-wider">Cerebras Swarm</h3>
                    <p className="text-2xl font-mono text-white mt-1">{stats.agentCount} Agents</p>
                </div>
                <Cpu className="text-indigo-500" />
            </div>
            <div className="mt-2 text-xs text-indigo-400 font-mono">Processing: {stats.tokensPerSecond.toLocaleString()} t/s</div>
        </div>
        <div className="bg-zinc-900 border border-zinc-800 p-4 rounded-lg">
            <div className="flex justify-between items-start">
                <div>
                    <h3 className="text-zinc-400 text-xs uppercase font-bold tracking-wider">Grits Red-Flags</h3>
                    <p className="text-2xl font-mono text-white mt-1">{stats.redFlagCount} Rejected</p>
                </div>
                <AlertTriangle className="text-amber-500" />
            </div>
             <div className="mt-2 text-xs text-amber-500 font-mono">{stats.redFlagCount > 0 ? 'Cycle Detected' : 'No Issues'}</div>
        </div>
        <div className="bg-zinc-900 border border-zinc-800 p-4 rounded-lg">
            <div className="flex justify-between items-start">
                <div>
                    <h3 className="text-zinc-400 text-xs uppercase font-bold tracking-wider">Shadow Commits</h3>
                    <p className="text-2xl font-mono text-white mt-1">{stats.shadowCommits}</p>
                </div>
                <GitCommit className="text-emerald-500" />
            </div>
             <div className="mt-2 text-xs text-emerald-500 font-mono">System 1 Velocity</div>
        </div>
      </div>

      {/* Voting Arena */}
      <div className="flex-1 bg-zinc-900 border border-zinc-800 rounded-lg flex flex-col min-h-[400px]">
        <div className="p-4 border-b border-zinc-800 flex justify-between items-center bg-zinc-900/50">
            <div>
                <h2 className="text-sm font-bold text-white">Live Voting Arena</h2>
                <p className="text-xs text-zinc-500">Task: Implement Claims Struct (Micro-Task #t2b)</p>
            </div>
            <span className="px-2 py-1 bg-indigo-500/10 text-indigo-400 text-xs border border-indigo-500/20 rounded font-mono animate-pulse hidden sm:inline-block">
                VOTING IN PROGRESS
            </span>
        </div>

        <div className="p-4 grid gap-4 overflow-y-auto">
            {candidates.map((candidate) => (
                <div key={candidate.id} className={`border rounded-lg p-4 transition-all ${
                    candidate.status === 'accepted' 
                    ? 'bg-emerald-950/10 border-emerald-500/50' 
                    : 'bg-zinc-950 border-zinc-800 opacity-60 hover:opacity-100'
                }`}>
                    <div className="flex justify-between items-start mb-3">
                        <div className="flex items-center gap-3">
                            <span className={`w-6 h-6 rounded-full flex items-center justify-center text-xs font-bold ${
                                candidate.status === 'accepted' ? 'bg-emerald-500 text-zinc-900' : 'bg-zinc-800 text-zinc-400'
                            }`}>
                                #{candidate.id}
                            </span>
                            <div className="flex flex-col">
                                <span className="text-xs font-mono text-zinc-400">Score: {candidate.score.toFixed(2)}</span>
                                {candidate.status === 'accepted' && <span className="text-[10px] text-emerald-400 font-bold uppercase">Consensus Winner</span>}
                            </div>
                        </div>
                        {candidate.status === 'accepted' ? (
                            <CheckCircle size={18} className="text-emerald-500" />
                        ) : (
                            <XCircle size={18} className="text-rose-500" />
                        )}
                    </div>
                    
                    {/* Code Snippet */}
                    <div className="bg-black rounded border border-zinc-800 p-3 mb-3">
                        <pre className="font-mono text-xs text-zinc-300 overflow-x-auto">
                            <code>{candidate.snippet}</code>
                        </pre>
                    </div>

                    {/* Red Flags */}
                    {candidate.redFlags.length > 0 && (
                        <div className="flex gap-2 flex-wrap">
                            {candidate.redFlags.map((flag, idx) => (
                                <span key={idx} className="flex items-center gap-1 px-2 py-1 bg-rose-500/10 text-rose-400 border border-rose-500/20 rounded text-[10px] uppercase font-bold tracking-wide">
                                    <AlertTriangle size={10} />
                                    {flag}
                                </span>
                            ))}
                        </div>
                    )}
                </div>
            ))}
        </div>
      </div>
    </div>
  );
};

export default ExecutionPanel;

