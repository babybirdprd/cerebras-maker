import { useEffect, useState } from 'react';
import { Cpu, AlertTriangle, CheckCircle, XCircle, GitCommit, Loader2 } from 'lucide-react';
import { getExecutionMetrics, getVotingState, ExecutionMetrics, VotingCandidate, VotingState } from '../tauri-api';
import { useMakerStore } from '../store/makerStore';

interface ExecutionPanelProps {
  isActive?: boolean;
}

const ExecutionPanel: React.FC = () => {
  const { currentView } = useMakerStore();
  const isActive = currentView === 'execution';
  const [metrics, setMetrics] = useState<ExecutionMetrics | null>(null);
  const [votingState, setVotingState] = useState<VotingState | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    // Initial load
    fetchData();

    // Poll for updates when active
    const interval = setInterval(fetchData, 2000);
    return () => clearInterval(interval);
  }, [isActive]);

  const fetchData = async () => {
    try {
      const [metricsData, votingData] = await Promise.all([
        getExecutionMetrics(),
        getVotingState(),
      ]);
      setMetrics(metricsData);
      setVotingState(votingData);
      setError(null);
    } catch (err) {
      setError('Backend not connected');
      // Set default empty state
      setMetrics({
        active_atoms: 0,
        total_atoms_spawned: 0,
        total_tokens: 0,
        tokens_per_second: 0,
        red_flag_count: 0,
        shadow_commits: 0,
        last_updated_ms: 0,
      });
      setVotingState({
        task_id: '',
        task_description: '',
        candidates: [],
        is_voting: false,
        winner_id: null,
      });
    } finally {
      setLoading(false);
    }
  };

  // Convert VotingCandidate to display format
  const candidates: VotingCandidate[] = votingState?.candidates || [];

  if (loading) {
    return (
      <div className="h-full flex items-center justify-center">
        <Loader2 className="w-8 h-8 animate-spin text-indigo-500" />
      </div>
    );
  }

  return (
    <div className="h-full flex flex-col gap-6 p-4 lg:p-6 overflow-y-auto">
      {/* Error Banner */}
      {error && (
        <div className="bg-amber-900/20 border border-amber-500/30 rounded-lg p-3 text-amber-400 text-sm">
          ⚠️ {error} - Showing default values
        </div>
      )}

      {/* Header Stats */}
      <div className="grid grid-cols-1 md:grid-cols-4 gap-4">
        <div className="bg-zinc-900 border border-zinc-800 p-4 rounded-lg relative overflow-hidden group">
          <div className="absolute inset-0 bg-linear-to-r from-indigo-500/10 to-transparent opacity-0 group-hover:opacity-100 transition-opacity"></div>
          <div className="flex justify-between items-start">
            <div>
              <h3 className="text-zinc-400 text-xs uppercase font-bold tracking-wider">Active Atoms</h3>
              <p className="text-2xl font-mono text-white mt-1">{metrics?.active_atoms || 0}</p>
            </div>
            <Cpu className={`text-indigo-500 ${(metrics?.active_atoms || 0) > 0 ? 'animate-pulse' : ''}`} />
          </div>
          <div className="mt-2 text-xs text-indigo-400 font-mono">
            Total spawned: {metrics?.total_atoms_spawned || 0}
          </div>
        </div>
        <div className="bg-zinc-900 border border-zinc-800 p-4 rounded-lg relative overflow-hidden group">
          <div className="absolute inset-0 bg-linear-to-r from-cyan-500/10 to-transparent opacity-0 group-hover:opacity-100 transition-opacity"></div>
          <div className="flex justify-between items-start">
            <div>
              <h3 className="text-zinc-400 text-xs uppercase font-bold tracking-wider">Token Throughput</h3>
              <p className="text-2xl font-mono text-white mt-1">{(metrics?.tokens_per_second || 0).toFixed(0)} t/s</p>
            </div>
            <Cpu className="text-cyan-500" />
          </div>
          <div className="mt-2 text-xs text-cyan-400 font-mono">
            Total: {((metrics?.total_tokens || 0) / 1000).toFixed(1)}K tokens
          </div>
        </div>
        <div className="bg-zinc-900 border border-zinc-800 p-4 rounded-lg">
          <div className="flex justify-between items-start">
            <div>
              <h3 className="text-zinc-400 text-xs uppercase font-bold tracking-wider">Grits Red-Flags</h3>
              <p className="text-2xl font-mono text-white mt-1">{metrics?.red_flag_count || 0} Rejected</p>
            </div>
            <AlertTriangle className={`text-amber-500 ${(metrics?.red_flag_count || 0) > 0 ? 'animate-bounce' : ''}`} />
          </div>
          <div className="mt-2 text-xs text-amber-500 font-mono">
            {(metrics?.red_flag_count || 0) > 0 ? 'Issues detected' : 'No issues'}
          </div>
        </div>
        <div className="bg-zinc-900 border border-zinc-800 p-4 rounded-lg">
          <div className="flex justify-between items-start">
            <div>
              <h3 className="text-zinc-400 text-xs uppercase font-bold tracking-wider">Shadow Commits</h3>
              <p className="text-2xl font-mono text-white mt-1">{metrics?.shadow_commits || 0}</p>
            </div>
            <GitCommit className="text-emerald-500" />
          </div>
          <div className="mt-2 text-xs text-emerald-500 font-mono">Transactional snapshots</div>
        </div>
      </div>

      {/* Voting Arena */}
      <div className="flex-1 bg-zinc-900 border border-zinc-800 rounded-lg flex flex-col min-h-[400px]">
        <div className="p-4 border-b border-zinc-800 flex justify-between items-center bg-zinc-900/50">
          <div>
            <h2 className="text-sm font-bold text-white">Live Voting Arena</h2>
            <p className="text-xs text-zinc-500">
              {votingState?.task_description
                ? `Task: ${votingState.task_description}`
                : 'No active voting session'
              }
            </p>
          </div>
          {votingState?.is_voting && (
            <span className="px-2 py-1 bg-indigo-500/10 text-indigo-400 text-xs border border-indigo-500/20 rounded font-mono animate-pulse hidden sm:inline-block">
              VOTING IN PROGRESS
            </span>
          )}
          {votingState?.winner_id && (
            <span className="px-2 py-1 bg-emerald-500/10 text-emerald-400 text-xs border border-emerald-500/20 rounded font-mono hidden sm:inline-block">
              CONSENSUS REACHED
            </span>
          )}
        </div>

        <div className="p-4 grid gap-4 overflow-y-auto flex-1">
          {candidates.length === 0 ? (
            <div className="flex flex-col items-center justify-center h-full text-zinc-500">
              <Cpu size={48} className="mb-4 opacity-30" />
              <p className="text-sm">No voting candidates yet</p>
              <p className="text-xs mt-1">Start a task to see real-time voting</p>
            </div>
          ) : (
            candidates.map((candidate) => (
              <div key={candidate.id} className={`border rounded-lg p-4 transition-all ${candidate.status === 'accepted'
                ? 'bg-emerald-950/10 border-emerald-500/50'
                : candidate.status === 'rejected'
                  ? 'bg-zinc-950 border-zinc-800 opacity-60'
                  : 'bg-zinc-950 border-indigo-500/30 hover:border-indigo-500/50'
                }`}>
                <div className="flex justify-between items-start mb-3">
                  <div className="flex items-center gap-3">
                    <span className={`w-6 h-6 rounded-full flex items-center justify-center text-xs font-bold ${candidate.status === 'accepted' ? 'bg-emerald-500 text-zinc-900' :
                      candidate.status === 'pending' ? 'bg-indigo-500 text-white' :
                        'bg-zinc-800 text-zinc-400'
                      }`}>
                      #{candidate.id}
                    </span>
                    <div className="flex flex-col">
                      <span className="text-xs font-mono text-zinc-400">
                        Score: {candidate.score.toFixed(2)} | Votes: {candidate.votes}
                      </span>
                      {candidate.status === 'accepted' && <span className="text-[10px] text-emerald-400 font-bold uppercase">Consensus Winner</span>}
                      {candidate.status === 'pending' && <span className="text-[10px] text-indigo-400 font-bold uppercase">Voting...</span>}
                    </div>
                  </div>
                  {candidate.status === 'accepted' ? (
                    <CheckCircle size={18} className="text-emerald-500" />
                  ) : candidate.status === 'pending' ? (
                    <Loader2 size={18} className="text-indigo-500 animate-spin" />
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
                {candidate.red_flags.length > 0 && (
                  <div className="flex gap-2 shrink-0">
                    {candidate.red_flags.map((flag, idx) => (
                      <span key={idx} className="flex items-center gap-1 px-2 py-1 bg-rose-500/10 text-rose-400 border border-rose-500/20 rounded text-[10px] uppercase font-bold tracking-wide">
                        <AlertTriangle size={10} />
                        {flag}
                      </span>
                    ))}
                  </div>
                )}
              </div>
            ))
          )}
        </div>
      </div>
    </div>
  );
};

export default ExecutionPanel;

