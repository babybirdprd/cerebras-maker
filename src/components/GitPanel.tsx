import { useState, useEffect, useCallback } from 'react';
import {
  GitBranch,
  GitCommit as GitCommitIcon,
  Cloud,
  Plus,
  Upload,
  RefreshCw,
  Loader2,
  AlertCircle,
  CheckCircle2,
  FileText,
  X,
  ChevronDown,
  Rocket,
  Settings,
  Check,
  Download,
} from 'lucide-react';
import {
  gitCurrentBranch,
  gitStatus,
  gitGetRemotes,
  gitAddRemote,
  gitPush,
  gitAdd,
  gitCommit,
  gitListBranches,
  gitPull,
  generateGithubWorkflow,
  generateDeployConfig,
  GitRemote,
  GitStatus,
  GitChange,
  GitBranchList,
  WorkflowConfig,
} from '../hooks/useTauri';

interface GitPanelProps {
  workspacePath: string;
  className?: string;
}

const GitPanel: React.FC<GitPanelProps> = ({ workspacePath, className = '' }) => {
  const [currentBranch, setCurrentBranch] = useState<string>('');
  const [status, setStatus] = useState<GitStatus | null>(null);
  const [remotes, setRemotes] = useState<GitRemote[]>([]);
  const [_branches, setBranches] = useState<GitBranchList | null>(null); // Reserved for future branch UI
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [successMessage, setSuccessMessage] = useState<string | null>(null);
  const [activeTab, setActiveTab] = useState<'git' | 'deploy'>('git');

  // Commit state
  const [selectedFiles, setSelectedFiles] = useState<string[]>([]);
  const [commitMessage, setCommitMessage] = useState('');
  const [isCommitting, setIsCommitting] = useState(false);

  // Add remote state
  const [showAddRemote, setShowAddRemote] = useState(false);
  const [newRemoteName, setNewRemoteName] = useState('origin');
  const [newRemoteUrl, setNewRemoteUrl] = useState('');
  const [isAddingRemote, setIsAddingRemote] = useState(false);

  // Push state
  const [selectedRemote, setSelectedRemote] = useState<string>('origin');
  const [pushBranch, setPushBranch] = useState<string>('');
  const [setUpstream, setSetUpstream] = useState(true);
  const [isPushing, setIsPushing] = useState(false);
  const [isPulling, setIsPulling] = useState(false);

  // Deployment state
  const [projectType, setProjectType] = useState<string>('tauri');
  const [deployTarget, setDeployTarget] = useState<string>('');
  const [runTests, setRunTests] = useState(true);
  const [runLint, setRunLint] = useState(true);
  const [isGenerating, setIsGenerating] = useState(false);

  const fetchGitInfo = useCallback(async () => {
    if (!workspacePath) return;
    setIsLoading(true);
    setError(null);
    try {
      const [branch, statusResult, remotesResult, branchList] = await Promise.all([
        gitCurrentBranch(workspacePath),
        gitStatus(workspacePath),
        gitGetRemotes(workspacePath),
        gitListBranches(workspacePath).catch(() => null),
      ]);
      setCurrentBranch(branch);
      setStatus(statusResult);
      setRemotes(remotesResult.remotes);
      setBranches(branchList);
      setPushBranch(branch);
      if (remotesResult.remotes.length > 0 && !remotesResult.remotes.find(r => r.name === selectedRemote)) {
        setSelectedRemote(remotesResult.remotes[0].name);
      }
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Failed to fetch git info');
    } finally {
      setIsLoading(false);
    }
  }, [workspacePath, selectedRemote]);

  useEffect(() => {
    fetchGitInfo();
  }, [fetchGitInfo]);

  const handleStageAndCommit = async () => {
    if (!commitMessage.trim()) return;
    setIsCommitting(true);
    setError(null);
    try {
      // Stage selected files or all if none selected
      const filesToStage = selectedFiles.length > 0 ? selectedFiles : ['.'];
      await gitAdd(workspacePath, filesToStage);

      // Commit
      const result = await gitCommit(workspacePath, commitMessage.trim());
      if (result.success) {
        setSuccessMessage(`Committed: ${result.commit_hash?.substring(0, 7)} - ${commitMessage}`);
        setCommitMessage('');
        setSelectedFiles([]);
        await fetchGitInfo();
      } else {
        setError(result.message);
      }
      setTimeout(() => setSuccessMessage(null), 3000);
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Failed to commit');
    } finally {
      setIsCommitting(false);
    }
  };

  const handleAddRemote = async () => {
    if (!newRemoteName.trim() || !newRemoteUrl.trim()) return;
    setIsAddingRemote(true);
    setError(null);
    try {
      await gitAddRemote(workspacePath, newRemoteName.trim(), newRemoteUrl.trim());
      setSuccessMessage(`Remote "${newRemoteName}" added successfully`);
      setShowAddRemote(false);
      setNewRemoteName('origin');
      setNewRemoteUrl('');
      await fetchGitInfo();
      setTimeout(() => setSuccessMessage(null), 3000);
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Failed to add remote');
    } finally {
      setIsAddingRemote(false);
    }
  };

  const handlePull = async () => {
    if (!selectedRemote || !pushBranch) return;
    setIsPulling(true);
    setError(null);
    try {
      await gitPull(workspacePath, selectedRemote, pushBranch, false);
      setSuccessMessage(`Pulled from ${selectedRemote}/${pushBranch}`);
      await fetchGitInfo();
      setTimeout(() => setSuccessMessage(null), 3000);
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Failed to pull');
    } finally {
      setIsPulling(false);
    }
  };

  const handlePush = async () => {
    if (!selectedRemote || !pushBranch) return;
    setIsPushing(true);
    setError(null);
    try {
      await gitPush(workspacePath, selectedRemote, pushBranch, setUpstream);
      setSuccessMessage(`Pushed to ${selectedRemote}/${pushBranch} successfully`);
      setTimeout(() => setSuccessMessage(null), 3000);
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Failed to push');
    } finally {
      setIsPushing(false);
    }
  };

  const getStatusIcon = (change: GitChange) => {
    switch (change.status) {
      case 'modified': return <FileText size={14} className="text-yellow-400" />;
      case 'added': return <Plus size={14} className="text-green-400" />;
      case 'deleted': return <X size={14} className="text-red-400" />;
      default: return <FileText size={14} className="text-zinc-400" />;
    }
  };

  const handleGenerateWorkflow = async () => {
    setIsGenerating(true);
    setError(null);
    try {
      const config: WorkflowConfig = {
        project_type: projectType,
        deploy_target: deployTarget || undefined,
        run_tests: runTests,
        run_lint: runLint,
      };
      const result = await generateGithubWorkflow(workspacePath, config);
      if (result.success) {
        setSuccessMessage(`Generated GitHub workflow at ${result.path}`);
      }
      setTimeout(() => setSuccessMessage(null), 5000);
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Failed to generate workflow');
    } finally {
      setIsGenerating(false);
    }
  };

  const handleGenerateDeployConfig = async () => {
    if (!deployTarget) return;
    setIsGenerating(true);
    setError(null);
    try {
      const result = await generateDeployConfig(workspacePath, deployTarget);
      if (result.success) {
        setSuccessMessage(`Generated ${deployTarget} config at ${result.path}`);
      }
      setTimeout(() => setSuccessMessage(null), 5000);
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Failed to generate config');
    } finally {
      setIsGenerating(false);
    }
  };

  const toggleFileSelection = (file: string) => {
    setSelectedFiles(prev =>
      prev.includes(file)
        ? prev.filter(f => f !== file)
        : [...prev, file]
    );
  };

  return (
    <div className={`bg-zinc-900 border border-zinc-700 rounded-xl p-6 ${className}`}>
      {/* Header */}
      <div className="flex items-center justify-between mb-4">
        <div className="flex items-center gap-2">
          <GitBranch className="text-indigo-400" size={20} />
          <h3 className="text-white font-medium">Git & Deploy</h3>
        </div>
        <button
          onClick={fetchGitInfo}
          disabled={isLoading}
          className="p-2 text-zinc-400 hover:text-white hover:bg-zinc-800 rounded-lg transition-colors"
        >
          <RefreshCw size={16} className={isLoading ? 'animate-spin' : ''} />
        </button>
      </div>

      {/* Tab Switcher */}
      <div className="flex gap-2 mb-4">
        <button
          onClick={() => setActiveTab('git')}
          className={`px-3 py-1.5 text-sm rounded-lg transition-colors ${
            activeTab === 'git'
              ? 'bg-indigo-600 text-white'
              : 'bg-zinc-800 text-zinc-400 hover:text-white'
          }`}
        >
          <GitBranch size={14} className="inline mr-1" /> Git
        </button>
        <button
          onClick={() => setActiveTab('deploy')}
          className={`px-3 py-1.5 text-sm rounded-lg transition-colors ${
            activeTab === 'deploy'
              ? 'bg-indigo-600 text-white'
              : 'bg-zinc-800 text-zinc-400 hover:text-white'
          }`}
        >
          <Rocket size={14} className="inline mr-1" /> Deploy
        </button>
      </div>

      {activeTab === 'git' && (
        <>
          {/* Current Branch */}
          <div className="bg-black border border-zinc-800 rounded-lg p-3 mb-4">
            <div className="flex items-center gap-2">
              <GitCommitIcon size={14} className="text-indigo-400" />
              <span className="text-zinc-400 text-sm">Current Branch:</span>
              <span className="text-white font-mono text-sm">{currentBranch || '—'}</span>
            </div>
          </div>

          {/* Status & Commit Section */}
          <div className="mb-4">
            <h4 className="text-sm font-medium text-zinc-300 mb-2">Changes & Commit</h4>
            {status ? (
              <div className={`bg-black border rounded-lg p-3 ${status.is_clean ? 'border-green-500/30' : 'border-yellow-500/30'}`}>
                <div className="flex items-center gap-2 mb-2">
                  {status.is_clean ? (
                    <>
                      <CheckCircle2 size={14} className="text-green-400" />
                      <span className="text-green-400 text-sm">Working tree clean</span>
                    </>
                  ) : (
                    <>
                      <AlertCircle size={14} className="text-yellow-400" />
                      <span className="text-yellow-400 text-sm">{status.change_count} changed file(s)</span>
                      <span className="text-zinc-500 text-xs ml-auto">
                        {selectedFiles.length > 0 ? `${selectedFiles.length} selected` : 'Click to select'}
                      </span>
                    </>
                  )}
                </div>
                {!status.is_clean && status.changes.length > 0 && (
                  <>
                    <div className="max-h-32 overflow-y-auto scrollbar-thin space-y-1 mb-3">
                      {status.changes.map((change, idx) => (
                        <div
                          key={idx}
                          onClick={() => toggleFileSelection(change.file)}
                          className={`flex items-center gap-2 text-xs p-1 rounded cursor-pointer transition-colors ${
                            selectedFiles.includes(change.file)
                              ? 'bg-indigo-500/20 border border-indigo-500/30'
                              : 'hover:bg-zinc-800'
                          }`}
                        >
                          {selectedFiles.includes(change.file) ? (
                            <Check size={14} className="text-indigo-400" />
                          ) : (
                            getStatusIcon(change)
                          )}
                          <span className="text-zinc-400 font-mono truncate">{change.file}</span>
                        </div>
                      ))}
                    </div>
                    {/* Commit Form */}
                    <div className="border-t border-zinc-800 pt-3 space-y-2">
                      <input
                        type="text"
                        value={commitMessage}
                        onChange={(e) => setCommitMessage(e.target.value)}
                        placeholder="Commit message..."
                        className="w-full bg-zinc-900 border border-zinc-700 rounded-lg px-3 py-2 text-white text-sm placeholder-zinc-500 focus:border-indigo-500 focus:outline-none"
                      />
                      <button
                        onClick={handleStageAndCommit}
                        disabled={isCommitting || !commitMessage.trim()}
                        className="w-full py-2 bg-green-600 hover:bg-green-500 disabled:bg-zinc-700 disabled:cursor-not-allowed text-white rounded-lg text-sm font-medium flex items-center justify-center gap-2"
                      >
                        {isCommitting ? <Loader2 size={14} className="animate-spin" /> : <GitCommitIcon size={14} />}
                        {selectedFiles.length > 0 ? `Commit ${selectedFiles.length} file(s)` : 'Commit All'}
                      </button>
                    </div>
                  </>
                )}
              </div>
            ) : (
              <div className="bg-black border border-zinc-800 rounded-lg p-3 text-zinc-500 text-sm">
                {isLoading ? 'Loading...' : 'No status available'}
              </div>
            )}
          </div>

      {/* Remotes Section */}
      <div className="mb-4">
        <div className="flex items-center justify-between mb-2">
          <h4 className="text-sm font-medium text-zinc-300">Remotes</h4>
          <button
            onClick={() => setShowAddRemote(!showAddRemote)}
            className="text-xs text-indigo-400 hover:text-indigo-300 flex items-center gap-1"
          >
            <Plus size={12} /> Add Remote
          </button>
        </div>

        {/* Remote List */}
        {remotes.length > 0 ? (
          <div className="bg-black border border-zinc-800 rounded-lg divide-y divide-zinc-800">
            {remotes.map((remote, idx) => (
              <div key={idx} className="p-3 flex items-center gap-2">
                <Cloud size={14} className="text-indigo-400" />
                <span className="text-white text-sm font-medium">{remote.name}</span>
                <span className="text-zinc-500 text-xs font-mono truncate flex-1">{remote.url}</span>
              </div>
            ))}
          </div>
        ) : (
          <div className="bg-black border border-zinc-800 rounded-lg p-3 text-zinc-500 text-sm">
            No remotes configured
          </div>
        )}

        {/* Add Remote Form */}
        {showAddRemote && (
          <div className="mt-3 bg-zinc-800/50 border border-zinc-700 rounded-lg p-4 space-y-3">
            <input
              type="text"
              value={newRemoteName}
              onChange={(e) => setNewRemoteName(e.target.value)}
              placeholder="Remote name (e.g., origin)"
              className="w-full bg-black border border-zinc-700 rounded-lg px-3 py-2 text-white text-sm placeholder-zinc-500 focus:border-indigo-500 focus:outline-none"
            />
            <input
              type="text"
              value={newRemoteUrl}
              onChange={(e) => setNewRemoteUrl(e.target.value)}
              placeholder="https://github.com/user/repo.git"
              className="w-full bg-black border border-zinc-700 rounded-lg px-3 py-2 text-white text-sm placeholder-zinc-500 focus:border-indigo-500 focus:outline-none"
            />
            <div className="flex gap-2">
              <button
                onClick={handleAddRemote}
                disabled={isAddingRemote || !newRemoteName.trim() || !newRemoteUrl.trim()}
                className="flex-1 py-2 bg-indigo-600 hover:bg-indigo-500 disabled:bg-zinc-700 disabled:cursor-not-allowed text-white rounded-lg text-sm font-medium flex items-center justify-center gap-2"
              >
                {isAddingRemote ? <Loader2 size={14} className="animate-spin" /> : <Plus size={14} />}
                Add
              </button>
              <button
                onClick={() => setShowAddRemote(false)}
                className="px-4 py-2 text-zinc-400 hover:text-white border border-zinc-700 hover:border-zinc-500 rounded-lg text-sm transition-colors"
              >
                Cancel
              </button>
            </div>
          </div>
        )}
      </div>

          {/* Sync Section */}
          <div className="mb-4">
            <h4 className="text-sm font-medium text-zinc-300 mb-2">Sync with Remote</h4>
            <div className="bg-black border border-zinc-800 rounded-lg p-4 space-y-3">
              <div className="grid grid-cols-2 gap-3">
                <div className="relative">
                  <label className="block text-xs text-zinc-500 mb-1">Remote</label>
                  <select
                    value={selectedRemote}
                    onChange={(e) => setSelectedRemote(e.target.value)}
                    disabled={remotes.length === 0}
                    className="w-full appearance-none bg-zinc-800 border border-zinc-700 rounded-lg px-3 py-2 pr-8 text-white text-sm focus:outline-none focus:border-indigo-500 disabled:opacity-50"
                  >
                    {remotes.map((r) => (
                      <option key={r.name} value={r.name}>{r.name}</option>
                    ))}
                  </select>
                  <ChevronDown size={14} className="absolute right-3 top-8 text-zinc-500 pointer-events-none" />
                </div>
                <div>
                  <label className="block text-xs text-zinc-500 mb-1">Branch</label>
                  <input
                    type="text"
                    value={pushBranch}
                    onChange={(e) => setPushBranch(e.target.value)}
                    placeholder="main"
                    className="w-full bg-zinc-800 border border-zinc-700 rounded-lg px-3 py-2 text-white text-sm focus:outline-none focus:border-indigo-500"
                  />
                </div>
              </div>
              <label className="flex items-center gap-2 cursor-pointer">
                <input
                  type="checkbox"
                  checked={setUpstream}
                  onChange={(e) => setSetUpstream(e.target.checked)}
                  className="w-4 h-4 rounded border-zinc-600 bg-zinc-800 text-indigo-500 focus:ring-indigo-500 focus:ring-offset-0"
                />
                <span className="text-zinc-400 text-xs">Set upstream (-u)</span>
              </label>
              <div className="flex gap-2">
                <button
                  onClick={handlePull}
                  disabled={isPulling || remotes.length === 0 || !pushBranch}
                  className="flex-1 py-2 bg-zinc-700 hover:bg-zinc-600 disabled:bg-zinc-800 disabled:cursor-not-allowed text-white rounded-lg font-medium flex items-center justify-center gap-2"
                >
                  {isPulling ? <Loader2 size={16} className="animate-spin" /> : <Download size={16} />}
                  Pull
                </button>
                <button
                  onClick={handlePush}
                  disabled={isPushing || remotes.length === 0 || !pushBranch}
                  className="flex-1 py-2 bg-indigo-600 hover:bg-indigo-500 disabled:bg-zinc-700 disabled:cursor-not-allowed text-white rounded-lg font-medium flex items-center justify-center gap-2"
                >
                  {isPushing ? <Loader2 size={16} className="animate-spin" /> : <Upload size={16} />}
                  Push
                </button>
              </div>
            </div>
          </div>
        </>
      )}

      {activeTab === 'deploy' && (
        <>
          {/* GitHub Actions Workflow Generator */}
          <div className="mb-4">
            <h4 className="text-sm font-medium text-zinc-300 mb-2">GitHub Actions Workflow</h4>
            <div className="bg-black border border-zinc-800 rounded-lg p-4 space-y-3">
              <div className="relative">
                <label className="block text-xs text-zinc-500 mb-1">Project Type</label>
                <select
                  value={projectType}
                  onChange={(e) => setProjectType(e.target.value)}
                  className="w-full appearance-none bg-zinc-800 border border-zinc-700 rounded-lg px-3 py-2 pr-8 text-white text-sm focus:outline-none focus:border-indigo-500"
                >
                  <option value="tauri">Tauri (Desktop App)</option>
                  <option value="react">React / Vite</option>
                  <option value="node">Node.js</option>
                  <option value="rust">Rust</option>
                </select>
                <ChevronDown size={14} className="absolute right-3 top-8 text-zinc-500 pointer-events-none" />
              </div>
              <div className="flex gap-4">
                <label className="flex items-center gap-2 cursor-pointer">
                  <input
                    type="checkbox"
                    checked={runTests}
                    onChange={(e) => setRunTests(e.target.checked)}
                    className="w-4 h-4 rounded border-zinc-600 bg-zinc-800 text-indigo-500"
                  />
                  <span className="text-zinc-400 text-xs">Run tests</span>
                </label>
                <label className="flex items-center gap-2 cursor-pointer">
                  <input
                    type="checkbox"
                    checked={runLint}
                    onChange={(e) => setRunLint(e.target.checked)}
                    className="w-4 h-4 rounded border-zinc-600 bg-zinc-800 text-indigo-500"
                  />
                  <span className="text-zinc-400 text-xs">Run lint</span>
                </label>
              </div>
              <button
                onClick={handleGenerateWorkflow}
                disabled={isGenerating}
                className="w-full py-2 bg-indigo-600 hover:bg-indigo-500 disabled:bg-zinc-700 disabled:cursor-not-allowed text-white rounded-lg font-medium flex items-center justify-center gap-2"
              >
                {isGenerating ? <Loader2 size={16} className="animate-spin" /> : <Settings size={16} />}
                Generate CI Workflow
              </button>
            </div>
          </div>

          {/* Deployment Config */}
          <div className="mb-4">
            <h4 className="text-sm font-medium text-zinc-300 mb-2">Deployment Configuration</h4>
            <div className="bg-black border border-zinc-800 rounded-lg p-4 space-y-3">
              <div className="relative">
                <label className="block text-xs text-zinc-500 mb-1">Deploy Target</label>
                <select
                  value={deployTarget}
                  onChange={(e) => setDeployTarget(e.target.value)}
                  className="w-full appearance-none bg-zinc-800 border border-zinc-700 rounded-lg px-3 py-2 pr-8 text-white text-sm focus:outline-none focus:border-indigo-500"
                >
                  <option value="">Select platform...</option>
                  <option value="vercel">Vercel</option>
                  <option value="netlify">Netlify</option>
                  <option value="github-pages">GitHub Pages</option>
                </select>
                <ChevronDown size={14} className="absolute right-3 top-8 text-zinc-500 pointer-events-none" />
              </div>
              <button
                onClick={handleGenerateDeployConfig}
                disabled={isGenerating || !deployTarget}
                className="w-full py-2 bg-green-600 hover:bg-green-500 disabled:bg-zinc-700 disabled:cursor-not-allowed text-white rounded-lg font-medium flex items-center justify-center gap-2"
              >
                {isGenerating ? <Loader2 size={16} className="animate-spin" /> : <Rocket size={16} />}
                Generate Deploy Config
              </button>
            </div>
          </div>

          {/* Deploy Info */}
          <div className="bg-zinc-800/50 border border-zinc-700 rounded-lg p-3">
            <p className="text-zinc-400 text-xs">
              <strong className="text-zinc-300">Tip:</strong> After generating configs, commit and push to trigger the workflow.
              Make sure to add the required secrets to your GitHub repository settings.
            </p>
          </div>
        </>
      )}

      {/* Error Display */}
      {error && (
        <div className="mt-4 p-3 bg-red-500/10 border border-red-500/30 rounded-lg flex items-start gap-2">
          <AlertCircle size={16} className="text-red-400 mt-0.5 flex-shrink-0" />
          <span className="text-red-300 text-sm">{error}</span>
        </div>
      )}

      {/* Success Display */}
      {successMessage && (
        <div className="mt-4 p-3 bg-green-500/10 border border-green-500/30 rounded-lg flex items-start gap-2">
          <CheckCircle2 size={16} className="text-green-400 mt-0.5 flex-shrink-0" />
          <span className="text-green-300 text-sm">{successMessage}</span>
        </div>
      )}
    </div>
  );
};

export default GitPanel;

