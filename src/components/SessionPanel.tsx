import { useState, useEffect, useCallback } from 'react';
import { Save, FolderOpen, Trash2, Clock, FileText, MessageSquare, Database, Loader2, Plus, RefreshCw, CheckCircle, AlertCircle, Zap } from 'lucide-react';
import {
  SessionSummary,
  saveSession,
  loadSession,
  listSessions,
  deleteSession,
} from '../tauri-api';
import { useMakerStore } from '../store/makerStore';

interface SessionPanelProps {
  className?: string;
}

const SessionPanel: React.FC<SessionPanelProps> = ({ className = '' }) => {
  const {
    workspacePath,
    prdFile,
    chatMessages,
    planContent,
    currentView,
    currentSessionId,
    autoSaveEnabled,
    autoSaveStatus,
    lastAutoSave,
    setCurrentSessionId,
    setAutoSaveEnabled,
    setPrdFile,
    setChatMessages,
    setPlanContent,
    setCurrentView,
    setWorkspacePath,
  } = useMakerStore();
  const [sessions, setSessions] = useState<SessionSummary[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [isSaving, setIsSaving] = useState(false);
  const [sessionName, setSessionName] = useState('');
  const [showNewSession, setShowNewSession] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [success, setSuccess] = useState<string | null>(null);

  const loadSessions = useCallback(async () => {
    setIsLoading(true);
    try {
      const list = await listSessions();
      setSessions(list);
      setError(null);
    } catch (e) {
      setError('Failed to load sessions');
      console.error('Failed to load sessions:', e);
    } finally {
      setIsLoading(false);
    }
  }, []);

  useEffect(() => {
    loadSessions();
  }, [loadSessions]);

  const handleSave = async () => {
    if (!sessionName.trim()) {
      setError('Please enter a session name');
      return;
    }

    setIsSaving(true);
    setError(null);
    try {
      const session = await saveSession(
        sessionName.trim(),
        workspacePath || '',
        prdFile?.content || null,
        prdFile?.name || null,
        chatMessages,
        planContent,
        currentView
      );
      // Notify parent to enable auto-save for this session
      setCurrentSessionId(session.id);
      setSuccess('Session saved! Auto-save enabled.');
      setSessionName('');
      setShowNewSession(false);
      await loadSessions();
      setTimeout(() => setSuccess(null), 3000);
    } catch (e) {
      setError(`Failed to save session: ${e}`);
    } finally {
      setIsSaving(false);
    }
  };

  const handleLoad = async (sessionId: string) => {
    setIsLoading(true);
    setError(null);
    try {
      const session = await loadSession(sessionId);
      // Simplified: Just update store values directly
      setWorkspacePath(session.workspace_path || '');
      setPrdFile({ content: session.prd_content || '', name: 'prd.md', type: 'md' });
      setChatMessages(session.conversation_history as any);
      setPlanContent(session.plan_content);
      setCurrentView(session.current_view as any);
      setCurrentSessionId(session.id);
      setSuccess('Session loaded successfully!');
      setTimeout(() => setSuccess(null), 3000);
    } catch (e) {
      setError(`Failed to load session: ${e}`);
    } finally {
      setIsLoading(false);
    }
  };

  const handleDelete = async (sessionId: string, sessionName: string) => {
    if (!confirm(`Delete session "${sessionName}"? This cannot be undone.`)) {
      return;
    }

    try {
      await deleteSession(sessionId);
      await loadSessions();
      setSuccess('Session deleted');
      setTimeout(() => setSuccess(null), 3000);
    } catch (e) {
      setError(`Failed to delete session: ${e}`);
    }
  };

  const formatDate = (ms: number) => {
    const date = new Date(ms);
    return date.toLocaleDateString('en-US', {
      month: 'short',
      day: 'numeric',
      hour: '2-digit',
      minute: '2-digit',
    });
  };

  const getRelativeTime = (ms: number) => {
    const diff = Date.now() - ms;
    const minutes = Math.floor(diff / 60000);
    if (minutes < 1) return 'Just now';
    if (minutes < 60) return `${minutes}m ago`;
    const hours = Math.floor(minutes / 60);
    if (hours < 24) return `${hours}h ago`;
    const days = Math.floor(hours / 24);
    return `${days}d ago`;
  };

  return (
    <div className={`h-full flex flex-col p-4 lg:p-6 ${className}`}>
      {/* Header */}
      <div className="mb-6 flex justify-between items-start">
        <div>
          <h2 className="text-xl font-bold text-white">Sessions</h2>
          <p className="text-zinc-400 text-sm mt-1">
            Save and restore your work across sessions
          </p>
        </div>
        <div className="flex gap-2">
          <button
            onClick={loadSessions}
            disabled={isLoading}
            className="p-2 bg-zinc-800 hover:bg-zinc-700 rounded-lg border border-zinc-700 transition-colors"
            title="Refresh"
          >
            <RefreshCw size={16} className={isLoading ? 'animate-spin' : ''} />
          </button>
          <button
            onClick={() => setShowNewSession(!showNewSession)}
            className="flex items-center gap-2 px-3 py-2 bg-indigo-600 hover:bg-indigo-500 text-white rounded-lg transition-colors"
          >
            <Plus size={16} />
            <span className="hidden sm:inline">Save Session</span>
          </button>
        </div>
      </div>

      {/* Auto-save Status Bar */}
      <div className="mb-4 p-3 bg-zinc-900 border border-zinc-800 rounded-lg flex items-center justify-between">
        <div className="flex items-center gap-3">
          <div className="flex items-center gap-2">
            <Zap size={14} className={autoSaveEnabled ? 'text-amber-400' : 'text-zinc-600'} />
            <span className="text-sm text-zinc-400">Auto-save</span>
          </div>
          <button
            onClick={() => setAutoSaveEnabled(!autoSaveEnabled)}
            className={`relative w-10 h-5 rounded-full transition-colors ${autoSaveEnabled ? 'bg-indigo-600' : 'bg-zinc-700'
              }`}
          >
            <div
              className={`absolute top-0.5 w-4 h-4 rounded-full bg-white transition-transform ${autoSaveEnabled ? 'left-5' : 'left-0.5'
                }`}
            />
          </button>
        </div>

        <div className="flex items-center gap-2 text-xs">
          {currentSessionId ? (
            <>
              {autoSaveStatus === 'saving' && (
                <span className="flex items-center gap-1 text-amber-400">
                  <Loader2 size={12} className="animate-spin" /> Saving...
                </span>
              )}
              {autoSaveStatus === 'saved' && (
                <span className="flex items-center gap-1 text-emerald-400">
                  <CheckCircle size={12} /> Saved
                </span>
              )}
              {autoSaveStatus === 'error' && (
                <span className="flex items-center gap-1 text-red-400">
                  <AlertCircle size={12} /> Error
                </span>
              )}
              {autoSaveStatus === 'idle' && lastAutoSave && (
                <span className="text-zinc-500">
                  Last: {lastAutoSave.toLocaleTimeString()}
                </span>
              )}
              {autoSaveStatus === 'idle' && !lastAutoSave && (
                <span className="text-zinc-500">Ready</span>
              )}
            </>
          ) : (
            <span className="text-zinc-500">Save a session to enable</span>
          )}
        </div>
      </div>

      {/* Status Messages */}
      {error && (
        <div className="mb-4 p-3 bg-red-900/20 border border-red-500/30 rounded-lg text-red-400 text-sm">
          {error}
        </div>
      )}
      {success && (
        <div className="mb-4 p-3 bg-emerald-900/20 border border-emerald-500/30 rounded-lg text-emerald-400 text-sm">
          {success}
        </div>
      )}

      {/* New Session Form */}
      {showNewSession && (
        <div className="mb-6 p-4 bg-zinc-900 border border-zinc-800 rounded-lg">
          <h3 className="text-sm font-medium text-white mb-3">Save Current Session</h3>
          <div className="flex gap-2">
            <input
              type="text"
              value={sessionName}
              onChange={(e) => setSessionName(e.target.value)}
              placeholder="Enter session name..."
              className="flex-1 px-3 py-2 bg-zinc-800 border border-zinc-700 rounded-lg text-white placeholder-zinc-500 text-sm focus:outline-none focus:border-indigo-500"
              onKeyDown={(e) => e.key === 'Enter' && handleSave()}
            />
            <button
              onClick={handleSave}
              disabled={isSaving || !sessionName.trim()}
              className="flex items-center gap-2 px-4 py-2 bg-indigo-600 hover:bg-indigo-500 disabled:bg-zinc-700 disabled:text-zinc-500 text-white rounded-lg transition-colors"
            >
              {isSaving ? <Loader2 size={16} className="animate-spin" /> : <Save size={16} />}
              Save
            </button>
          </div>
          <div className="mt-3 flex flex-wrap gap-2 text-xs text-zinc-500">
            <span className="flex items-center gap-1">
              <FileText size={12} /> PRD: {prdFile?.content ? 'Yes' : 'No'}
            </span>
            <span className="flex items-center gap-1">
              <MessageSquare size={12} /> Messages: {chatMessages.length}
            </span>
            <span className="flex items-center gap-1">
              <Database size={12} /> Plan: {planContent ? 'Yes' : 'No'}
            </span>
          </div>
        </div>
      )}

      {/* Sessions List */}
      <div className="flex-1 overflow-y-auto">
        {isLoading && sessions.length === 0 ? (
          <div className="flex items-center justify-center h-48">
            <Loader2 className="w-8 h-8 animate-spin text-indigo-500" />
          </div>
        ) : sessions.length === 0 ? (
          <div className="flex flex-col items-center justify-center h-48 text-zinc-500">
            <FolderOpen size={48} className="mb-4 opacity-30" />
            <p className="text-sm font-medium">No saved sessions</p>
            <p className="text-xs mt-1">Save your current work to resume later</p>
          </div>
        ) : (
          <div className="space-y-3">
            {sessions.map((session) => (
              <div
                key={session.id}
                className="bg-zinc-900 border border-zinc-800 rounded-lg p-4 hover:border-zinc-700 transition-colors group"
              >
                <div className="flex justify-between items-start mb-2">
                  <div className="flex-1 min-w-0">
                    <h3 className="font-medium text-white truncate">{session.name}</h3>
                    <p className="text-xs text-zinc-500 truncate">{session.workspace_path}</p>
                  </div>
                  <div className="flex items-center gap-1 opacity-0 group-hover:opacity-100 transition-opacity">
                    <button
                      onClick={() => handleLoad(session.id)}
                      className="p-2 bg-indigo-600 hover:bg-indigo-500 rounded-lg text-white transition-colors"
                      title="Load session"
                    >
                      <FolderOpen size={14} />
                    </button>
                    <button
                      onClick={() => handleDelete(session.id, session.name)}
                      className="p-2 bg-zinc-800 hover:bg-red-600 rounded-lg text-zinc-400 hover:text-white transition-colors"
                      title="Delete session"
                    >
                      <Trash2 size={14} />
                    </button>
                  </div>
                </div>

                <div className="flex flex-wrap gap-3 text-xs text-zinc-400">
                  <span className="flex items-center gap-1">
                    <Clock size={12} />
                    {getRelativeTime(session.updated_at_ms)}
                  </span>
                  {session.has_prd && (
                    <span className="flex items-center gap-1 text-emerald-400">
                      <FileText size={12} /> PRD
                    </span>
                  )}
                  {session.has_plan && (
                    <span className="flex items-center gap-1 text-indigo-400">
                      <Database size={12} /> Plan
                    </span>
                  )}
                  {session.message_count > 0 && (
                    <span className="flex items-center gap-1">
                      <MessageSquare size={12} /> {session.message_count} msgs
                    </span>
                  )}
                  {session.kb_document_count > 0 && (
                    <span className="flex items-center gap-1 text-amber-400">
                      <Database size={12} /> {session.kb_document_count} docs
                    </span>
                  )}
                </div>

                <div className="mt-2 text-[10px] text-zinc-600">
                  Created: {formatDate(session.created_at_ms)} • Updated: {formatDate(session.updated_at_ms)}
                </div>
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  );
};

export default SessionPanel;

