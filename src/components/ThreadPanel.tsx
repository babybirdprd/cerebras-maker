import React, { useState, useRef, useEffect, useCallback } from 'react';
import {
  MessageSquare,
  Send,
  Paperclip,
  CheckCircle2,
  AlertCircle,
  Clock,
  Bot,
  User,
  Filter,
  Plus,
  X,
  Link,
  FileCode,
  Loader2,
} from 'lucide-react';
import {
  Thread,
  ThreadAttachment,
  ThreadStatus,
  ThreadPriority,
} from '../types';
import {
  listThreads,
  replyToThread,
  resolveThread as resolveThreadApi,
} from '../tauri-api';

interface ThreadPanelProps {
  className?: string;
}

// Sample threads for demo when backend is empty
const DEMO_THREADS: Thread[] = [
  {
    id: 'demo-1',
    type: 'help_request',
    status: 'open',
    priority: 'high',
    title: 'Cannot find authentication documentation',
    agent_name: 'Research Agent',
    is_blocking: true,
    created_at: new Date(),
    updated_at: new Date(),
    messages: [
      {
        id: 'm1',
        thread_id: 'demo-1',
        role: 'agent',
        agent_name: 'Research Agent',
        content: "I need help finding documentation for the OAuth2 authentication flow. I searched the codebase but couldn't find any existing implementation.",
        timestamp: new Date(),
      },
    ],
  },
  {
    id: 'demo-2',
    type: 'clarification',
    status: 'pending',
    priority: 'medium',
    title: 'Database schema clarification needed',
    agent_name: 'Architect Agent',
    is_blocking: false,
    created_at: new Date(Date.now() - 3600000),
    updated_at: new Date(Date.now() - 1800000),
    messages: [
      {
        id: 'm2',
        thread_id: 'demo-2',
        role: 'agent',
        agent_name: 'Architect Agent',
        content: 'Should the users table include a soft-delete column or should we use a separate archive table?',
        timestamp: new Date(Date.now() - 3600000),
      },
      {
        id: 'm3',
        thread_id: 'demo-2',
        role: 'human',
        content: 'Use soft-delete with a deleted_at timestamp column.',
        timestamp: new Date(Date.now() - 1800000),
      },
    ],
  },
  {
    id: 'demo-3',
    type: 'approval_needed',
    status: 'resolved',
    priority: 'low',
    title: 'API endpoint naming convention',
    agent_name: 'Coder Agent',
    is_blocking: false,
    created_at: new Date(Date.now() - 86400000),
    updated_at: new Date(Date.now() - 43200000),
    resolved_at: new Date(Date.now() - 43200000),
    messages: [
      {
        id: 'm4',
        thread_id: 'demo-3',
        role: 'agent',
        agent_name: 'Coder Agent',
        content: 'Should I use camelCase or snake_case for the API response fields?',
        timestamp: new Date(Date.now() - 86400000),
      },
      {
        id: 'm5',
        thread_id: 'demo-3',
        role: 'human',
        content: 'Use snake_case to match our existing API conventions.',
        attachments: [
          { type: 'link', title: 'API Style Guide', url: 'https://docs.example.com/api-style' },
        ],
        timestamp: new Date(Date.now() - 43200000),
      },
    ],
  },
  {
    id: 'demo-4',
    type: 'resource_needed',
    status: 'open',
    priority: 'urgent',
    title: 'Missing AWS credentials for S3 upload',
    agent_name: 'Deployment Agent',
    is_blocking: true,
    created_at: new Date(Date.now() - 600000),
    updated_at: new Date(Date.now() - 600000),
    messages: [
      {
        id: 'm6',
        thread_id: 'demo-4',
        role: 'agent',
        agent_name: 'Deployment Agent',
        content: 'I cannot proceed with the file upload feature. The AWS S3 credentials are missing from the environment configuration.',
        timestamp: new Date(Date.now() - 600000),
      },
    ],
  },
];

const ThreadPanel: React.FC<ThreadPanelProps> = ({ className = '' }) => {
  const [threads, setThreads] = useState<Thread[]>(DEMO_THREADS);
  const [loading, setLoading] = useState(false);
  const [selectedThreadId, setSelectedThreadId] = useState<string | null>('demo-1');
  const [statusFilter, setStatusFilter] = useState<ThreadStatus | 'all'>('all');
  const [replyText, setReplyText] = useState('');
  const [attachments, setAttachments] = useState<ThreadAttachment[]>([]);
  const [showAttachmentMenu, setShowAttachmentMenu] = useState(false);
  const [sending, setSending] = useState(false);
  const messagesEndRef = useRef<HTMLDivElement>(null);

  // Load threads from backend
  const loadThreads = useCallback(async () => {
    setLoading(true);
    try {
      const backendThreads = await listThreads();
      // If backend has threads, use them; otherwise keep demo threads
      if (backendThreads.length > 0) {
        setThreads(backendThreads);
        if (!backendThreads.find(t => t.id === selectedThreadId)) {
          setSelectedThreadId(backendThreads[0]?.id ?? null);
        }
      }
    } catch (e) {
      console.log('Using demo threads:', e);
    }
    setLoading(false);
  }, [selectedThreadId]);

  useEffect(() => {
    loadThreads();
  }, [loadThreads]);

  const selectedThread = threads.find((t) => t.id === selectedThreadId);

  useEffect(() => {
    messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' });
  }, [selectedThread?.messages]);

  const filteredThreads = threads.filter((t) => statusFilter === 'all' || t.status === statusFilter);

  const getPriorityColor = (priority: ThreadPriority) => {
    switch (priority) {
      case 'urgent': return 'bg-rose-500/20 text-rose-400 border-rose-500/30';
      case 'high': return 'bg-amber-500/20 text-amber-400 border-amber-500/30';
      case 'medium': return 'bg-indigo-500/20 text-indigo-400 border-indigo-500/30';
      case 'low': return 'bg-zinc-500/20 text-zinc-400 border-zinc-500/30';
    }
  };

  const getStatusIcon = (status: ThreadStatus) => {
    switch (status) {
      case 'open': return <AlertCircle size={14} className="text-rose-400" />;
      case 'pending': return <Clock size={14} className="text-amber-400" />;
      case 'resolved': return <CheckCircle2 size={14} className="text-emerald-400" />;
    }
  };

  const getStatusColor = (status: ThreadStatus) => {
    switch (status) {
      case 'open': return 'text-rose-400';
      case 'pending': return 'text-amber-400';
      case 'resolved': return 'text-emerald-400';
    }
  };

  const formatTime = (date: Date) => {
    const now = new Date();
    const diff = now.getTime() - date.getTime();
    const minutes = Math.floor(diff / 60000);
    if (minutes < 1) return 'Just now';
    if (minutes < 60) return `${minutes}m ago`;
    const hours = Math.floor(minutes / 60);
    if (hours < 24) return `${hours}h ago`;
    const days = Math.floor(hours / 24);
    return `${days}d ago`;
  };

  const handleSendReply = async () => {
    if (!replyText.trim() || !selectedThreadId || sending) return;

    setSending(true);
    const messageContent = replyText.trim();
    const messageAttachments = attachments.length > 0 ? attachments : undefined;

    // Clear input immediately for better UX
    setReplyText('');
    setAttachments([]);

    try {
      // Try backend first (for non-demo threads)
      if (!selectedThreadId.startsWith('demo-')) {
        const updatedThread = await replyToThread(selectedThreadId, messageContent, messageAttachments);
        setThreads((prev) =>
          prev.map((t) => (t.id === selectedThreadId ? updatedThread : t))
        );
      } else {
        // Demo thread - update locally
        const newMessage = {
          id: `m${Date.now()}`,
          thread_id: selectedThreadId,
          role: 'human' as const,
          content: messageContent,
          attachments: messageAttachments,
          timestamp: new Date(),
        };
        setThreads((prev) =>
          prev.map((t) =>
            t.id === selectedThreadId
              ? {
                ...t,
                messages: [...t.messages, newMessage],
                updated_at: new Date(),
                status: 'pending' as ThreadStatus,
                is_blocking: false,
              }
              : t
          )
        );
      }
    } catch (e) {
      console.error('Failed to send reply:', e);
      // Restore the message on error
      setReplyText(messageContent);
      setAttachments(messageAttachments ?? []);
    }
    setSending(false);
  };

  const handleResolveThread = async () => {
    if (!selectedThreadId) return;

    try {
      // Try backend first (for non-demo threads)
      if (!selectedThreadId.startsWith('demo-')) {
        const updatedThread = await resolveThreadApi(selectedThreadId);
        setThreads((prev) =>
          prev.map((t) => (t.id === selectedThreadId ? updatedThread : t))
        );
      } else {
        // Demo thread - update locally
        setThreads((prev) =>
          prev.map((t) =>
            t.id === selectedThreadId
              ? {
                ...t,
                status: 'resolved' as ThreadStatus,
                resolved_at: new Date(),
                updated_at: new Date(),
                is_blocking: false,
              }
              : t
          )
        );
      }
    } catch (e) {
      console.error('Failed to resolve thread:', e);
    }
  };

  const addAttachment = (type: 'link' | 'code_snippet') => {
    const title = prompt(`Enter ${type === 'link' ? 'link title' : 'snippet title'}:`);
    if (!title) return;

    const content = prompt(type === 'link' ? 'Enter URL:' : 'Enter code snippet:');
    if (!content) return;

    setAttachments((prev) => [
      ...prev,
      {
        type,
        title,
        ...(type === 'link' ? { url: content } : { content }),
      },
    ]);
    setShowAttachmentMenu(false);
  };

  const removeAttachment = (index: number) => {
    setAttachments((prev) => prev.filter((_, i) => i !== index));
  };

  return (
    <div className={`h-full flex bg-zinc-950 ${className}`}>
      {/* Thread List - Left Sidebar */}
      <div className="w-80 border-r border-zinc-800 flex flex-col bg-zinc-900">
        {/* Filter Bar */}
        <div className="p-4 border-b border-zinc-800">
          <div className="flex items-center justify-between mb-3">
            <h2 className="text-lg font-bold text-white flex items-center gap-2">
              {loading ? (
                <Loader2 size={20} className="text-indigo-400 animate-spin" />
              ) : (
                <MessageSquare size={20} className="text-indigo-400" />
              )}
              Threads
            </h2>
            <button
              onClick={loadThreads}
              disabled={loading}
              className="p-2 bg-indigo-600 hover:bg-indigo-500 rounded-lg transition-colors disabled:opacity-50"
            >
              <Plus size={16} className="text-white" />
            </button>
          </div>

          <div className="flex gap-1">
            {(['all', 'open', 'pending', 'resolved'] as const).map((status) => (
              <button
                key={status}
                onClick={() => setStatusFilter(status)}
                className={`px-3 py-1.5 text-xs rounded-lg transition-colors ${statusFilter === status
                  ? 'bg-indigo-500/20 text-indigo-400 border border-indigo-500/30'
                  : 'bg-zinc-800 text-zinc-400 hover:bg-zinc-700 border border-transparent'
                  }`}
              >
                {status === 'all' ? 'All' : status.charAt(0).toUpperCase() + status.slice(1)}
              </button>
            ))}
          </div>
        </div>

        {/* Thread List */}
        <div className="flex-1 overflow-y-auto">
          {filteredThreads.length === 0 ? (
            <div className="p-8 text-center text-zinc-500">
              <Filter size={32} className="mx-auto mb-2 opacity-30" />
              <p className="text-sm">No threads found</p>
            </div>
          ) : (
            filteredThreads.map((thread) => (
              <button
                key={thread.id}
                onClick={() => setSelectedThreadId(thread.id)}
                className={`w-full p-4 text-left border-b border-zinc-800 transition-colors ${selectedThreadId === thread.id
                  ? 'bg-indigo-500/10 border-l-2 border-l-indigo-500'
                  : 'hover:bg-zinc-800/50'
                  }`}
              >
                <div className="flex items-start justify-between gap-2 mb-2">
                  <div className="flex items-center gap-2 min-w-0">
                    {getStatusIcon(thread.status)}
                    <span className="text-sm font-medium text-white truncate">
                      {thread.title}
                    </span>
                  </div>
                  {thread.is_blocking && (
                    <span className="shrink-0 px-2 py-0.5 text-[10px] font-medium bg-rose-500/20 text-rose-400 rounded-full border border-rose-500/30">
                      BLOCKING
                    </span>
                  )}
                </div>

                <div className="flex items-center gap-2 mb-2">
                  <span className={`px-2 py-0.5 text-[10px] font-medium rounded border ${getPriorityColor(thread.priority)}`}>
                    {thread.priority.toUpperCase()}
                  </span>
                  <span className="text-xs text-zinc-500">{thread.agent_name}</span>
                </div>

                <div className="flex items-center justify-between text-xs text-zinc-500">
                  <span>{thread.messages.length} message{thread.messages.length !== 1 ? 's' : ''}</span>
                  <span>{formatTime(thread.updated_at)}</span>
                </div>
              </button>
            ))
          )}
        </div>
      </div>

      {/* Thread Detail - Main Panel */}
      <div className="flex-1 flex flex-col">
        {selectedThread ? (
          <>
            {/* Thread Header */}
            <div className="p-4 border-b border-zinc-800 bg-zinc-900/50">
              <div className="flex items-start justify-between mb-3">
                <div className="flex-1 min-w-0">
                  <div className="flex items-center gap-2 mb-2">
                    {getStatusIcon(selectedThread.status)}
                    <h3 className="text-lg font-bold text-white truncate">{selectedThread.title}</h3>
                  </div>
                  <div className="flex items-center gap-3 text-sm">
                    <span className="flex items-center gap-1 text-zinc-400">
                      <Bot size={14} className="text-indigo-400" />
                      {selectedThread.agent_name}
                    </span>
                    <span className={`px-2 py-0.5 text-xs font-medium rounded border ${getPriorityColor(selectedThread.priority)}`}>
                      {selectedThread.priority.toUpperCase()}
                    </span>
                    <span className={`text-xs ${getStatusColor(selectedThread.status)}`}>
                      {selectedThread.status.charAt(0).toUpperCase() + selectedThread.status.slice(1)}
                    </span>
                  </div>
                </div>

                {selectedThread.is_blocking && (
                  <div className="shrink-0 px-3 py-1.5 bg-rose-500/20 border border-rose-500/30 rounded-lg flex items-center gap-2">
                    <AlertCircle size={14} className="text-rose-400" />
                    <span className="text-xs font-medium text-rose-400">Agent Blocked</span>
                  </div>
                )}

                {selectedThread.status !== 'resolved' && (
                  <button
                    onClick={handleResolveThread}
                    className="ml-3 px-3 py-1.5 bg-emerald-600 hover:bg-emerald-500 text-white text-xs font-medium rounded-lg transition-colors flex items-center gap-1"
                  >
                    <CheckCircle2 size={14} />
                    Resolve
                  </button>
                )}
              </div>
            </div>

            {/* Messages */}
            <div className="flex-1 overflow-y-auto p-4 space-y-4">
              {selectedThread.messages.map((msg) => (
                <div
                  key={msg.id}
                  className={`flex gap-3 ${msg.role === 'human' ? 'flex-row-reverse' : ''}`}
                >
                  <div className={`w-8 h-8 rounded-lg flex items-center justify-center shrink-0 ${msg.role === 'agent' ? 'bg-indigo-600' : 'bg-zinc-700'
                    }`}>
                    {msg.role === 'agent' ? <Bot size={16} className="text-white" /> : <User size={16} className="text-white" />}
                  </div>

                  <div className={`max-w-[70%] ${msg.role === 'human' ? 'items-end' : 'items-start'}`}>
                    {msg.role === 'agent' && msg.agent_name && (
                      <span className="text-xs text-indigo-400 mb-1 block">{msg.agent_name}</span>
                    )}

                    <div className={`rounded-xl px-4 py-3 ${msg.role === 'agent'
                      ? 'bg-zinc-800 border border-zinc-700'
                      : 'bg-indigo-600'
                      }`}>
                      <p className="text-sm text-white whitespace-pre-wrap">{msg.content}</p>

                      {msg.attachments && msg.attachments.length > 0 && (
                        <div className="mt-3 space-y-2">
                          {msg.attachments.map((att, i) => (
                            <div key={i} className="flex items-center gap-2 p-2 bg-zinc-900/50 rounded-lg">
                              {att.type === 'link' ? <Link size={12} className="text-zinc-400" /> : <FileCode size={12} className="text-zinc-400" />}
                              <span className="text-xs text-zinc-300">{att.title}</span>
                            </div>
                          ))}
                        </div>
                      )}
                    </div>

                    <span className="text-[10px] text-zinc-500 mt-1 block">
                      {msg.timestamp.toLocaleTimeString()}
                    </span>
                  </div>
                </div>
              ))}
              <div ref={messagesEndRef} />
            </div>

            {/* Reply Input */}
            {selectedThread.status !== 'resolved' && (
              <div className="p-4 border-t border-zinc-800 bg-zinc-900/50">
                {/* Attachments Preview */}
                {attachments.length > 0 && (
                  <div className="mb-3 flex flex-wrap gap-2">
                    {attachments.map((att, i) => (
                      <div key={i} className="flex items-center gap-2 px-2 py-1 bg-zinc-800 rounded-lg border border-zinc-700">
                        {att.type === 'link' ? <Link size={12} className="text-zinc-400" /> : <FileCode size={12} className="text-zinc-400" />}
                        <span className="text-xs text-zinc-300">{att.title}</span>
                        <button onClick={() => removeAttachment(i)} className="p-0.5 hover:bg-zinc-700 rounded">
                          <X size={12} className="text-zinc-500" />
                        </button>
                      </div>
                    ))}
                  </div>
                )}

                <div className="flex gap-3">
                  {/* Attachment Button */}
                  <div className="relative">
                    <button
                      onClick={() => setShowAttachmentMenu(!showAttachmentMenu)}
                      className="p-3 bg-zinc-800 hover:bg-zinc-700 border border-zinc-700 rounded-lg transition-colors"
                    >
                      <Paperclip size={18} className="text-zinc-400" />
                    </button>

                    {showAttachmentMenu && (
                      <div className="absolute bottom-full left-0 mb-2 bg-zinc-800 border border-zinc-700 rounded-lg shadow-xl py-1 min-w-[140px]">
                        <button
                          onClick={() => addAttachment('link')}
                          className="w-full px-3 py-2 text-left text-sm text-zinc-300 hover:bg-zinc-700 flex items-center gap-2"
                        >
                          <Link size={14} /> Add Link
                        </button>
                        <button
                          onClick={() => addAttachment('code_snippet')}
                          className="w-full px-3 py-2 text-left text-sm text-zinc-300 hover:bg-zinc-700 flex items-center gap-2"
                        >
                          <FileCode size={14} /> Code Snippet
                        </button>
                      </div>
                    )}
                  </div>

                  {/* Input */}
                  <textarea
                    value={replyText}
                    onChange={(e) => setReplyText(e.target.value)}
                    onKeyDown={(e) => {
                      if (e.key === 'Enter' && !e.shiftKey) {
                        e.preventDefault();
                        handleSendReply();
                      }
                    }}
                    placeholder="Type your response to unblock the agent..."
                    rows={1}
                    className="flex-1 bg-zinc-900 border border-zinc-700 rounded-lg px-4 py-3 text-sm text-white placeholder-zinc-500 focus:outline-none focus:border-indigo-500 resize-none"
                  />

                  {/* Send Button */}
                  <button
                    onClick={handleSendReply}
                    disabled={!replyText.trim() || sending}
                    className="px-4 bg-indigo-600 hover:bg-indigo-500 disabled:bg-zinc-700 disabled:cursor-not-allowed text-white rounded-lg transition-colors"
                  >
                    {sending ? <Loader2 size={18} className="animate-spin" /> : <Send size={18} />}
                  </button>
                </div>
              </div>
            )}

            {/* Resolved Notice */}
            {selectedThread.status === 'resolved' && (
              <div className="p-4 border-t border-zinc-800 bg-emerald-900/10">
                <div className="flex items-center justify-center gap-2 text-emerald-400">
                  <CheckCircle2 size={16} />
                  <span className="text-sm">This thread has been resolved</span>
                  {selectedThread.resolved_at && (
                    <span className="text-xs text-zinc-500 ml-2">
                      {formatTime(selectedThread.resolved_at)}
                    </span>
                  )}
                </div>
              </div>
            )}
          </>
        ) : (
          /* No Thread Selected */
          <div className="flex-1 flex flex-col items-center justify-center text-zinc-500">
            <MessageSquare size={48} className="mb-4 opacity-30" />
            <p className="text-lg font-medium">No thread selected</p>
            <p className="text-sm mt-1">Select a thread from the sidebar to view details</p>
          </div>
        )}
      </div>
    </div>
  );
};

export default ThreadPanel;
