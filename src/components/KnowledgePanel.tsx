import { useState, useCallback, useRef, useEffect } from 'react';
import { FileText, Trash2, Eye, Upload, Book, Globe, X, Loader2, Sparkles, BarChart3 } from 'lucide-react';
import {
  KnowledgeDocument,
  KnowledgeBaseStats,
  kbAddDocument,
  kbAddDocumentAuto,
  kbRemoveDocument,
  kbGetDocuments,
  kbCompileContext,
  kbGetStats,
} from '../hooks/useTauri';

interface KnowledgePanelProps {
  onContextChange?: (context: string) => void;
  className?: string;
}

const DOC_TYPES = [
  { id: 'auto', label: '✨ Auto-Detect' },
  { id: 'prd', label: 'PRD' },
  { id: 'api_reference', label: 'API Reference' },
  { id: 'design_spec', label: 'Design Spec' },
  { id: 'architecture', label: 'Architecture' },
  { id: 'style_guide', label: 'Style Guide' },
  { id: 'other', label: 'Other' },
];

const KnowledgePanel: React.FC<KnowledgePanelProps> = ({ onContextChange, className = '' }) => {
  const [documents, setDocuments] = useState<KnowledgeDocument[]>([]);
  const [stats, setStats] = useState<KnowledgeBaseStats | null>(null);
  const [isLoading, setIsLoading] = useState(false);
  const [isDragging, setIsDragging] = useState(false);
  const [selectedDocType, setSelectedDocType] = useState('auto');
  const [showPreview, setShowPreview] = useState(false);
  const [previewContent, setPreviewContent] = useState('');
  const [showStats, setShowStats] = useState(false);
  const fileInputRef = useRef<HTMLInputElement>(null);

  const loadDocuments = useCallback(async () => {
    try {
      const [docs, kbStats] = await Promise.all([
        kbGetDocuments(),
        kbGetStats()
      ]);
      setDocuments(docs);
      setStats(kbStats);
    } catch (e) {
      console.error('Failed to load documents:', e);
    }
  }, []);

  useEffect(() => {
    loadDocuments();
  }, [loadDocuments]);

  const handleDragOver = useCallback((e: React.DragEvent) => {
    e.preventDefault();
    setIsDragging(true);
  }, []);

  const handleDragLeave = useCallback((e: React.DragEvent) => {
    e.preventDefault();
    setIsDragging(false);
  }, []);

  const processFiles = useCallback(async (files: FileList) => {
    setIsLoading(true);
    for (const file of Array.from(files)) {
      try {
        const content = await file.text();
        if (selectedDocType === 'auto') {
          // Use auto-classification
          const result = await kbAddDocumentAuto(file.name, content);
          console.log(`Auto-classified "${file.name}" as ${result.doc_type}`);
        } else {
          await kbAddDocument(file.name, content, selectedDocType);
        }
      } catch (e) {
        console.error('Failed to add document:', e);
      }
    }
    await loadDocuments();
    setIsLoading(false);
  }, [selectedDocType, loadDocuments]);

  const handleDrop = useCallback((e: React.DragEvent) => {
    e.preventDefault();
    setIsDragging(false);
    if (e.dataTransfer.files.length > 0) {
      processFiles(e.dataTransfer.files);
    }
  }, [processFiles]);

  const handleFileSelect = useCallback((e: React.ChangeEvent<HTMLInputElement>) => {
    if (e.target.files && e.target.files.length > 0) {
      processFiles(e.target.files);
    }
  }, [processFiles]);

  const handleRemoveDocument = async (id: string) => {
    try {
      await kbRemoveDocument(id);
      await loadDocuments();
    } catch (e) {
      console.error('Failed to remove document:', e);
    }
  };

  const handlePreviewContext = async () => {
    setIsLoading(true);
    try {
      const context = await kbCompileContext();
      setPreviewContent(context);
      setShowPreview(true);
      onContextChange?.(context);
    } catch (e) {
      console.error('Failed to compile context:', e);
    }
    setIsLoading(false);
  };



  const getDocTypeBadgeColor = (docType: string) => {
    const colors: Record<string, string> = {
      prd: 'bg-indigo-500/20 text-indigo-300',
      api_reference: 'bg-green-500/20 text-green-300',
      design_spec: 'bg-purple-500/20 text-purple-300',
      architecture: 'bg-orange-500/20 text-orange-300',
      style_guide: 'bg-pink-500/20 text-pink-300',
      web_research: 'bg-cyan-500/20 text-cyan-300',
      other: 'bg-zinc-500/20 text-zinc-300',
    };
    return colors[docType] || colors.other;
  };

  return (
    <div className={`bg-zinc-900 border border-zinc-700 rounded-xl p-6 ${className}`}>
      <div className="flex items-center justify-between mb-4">
        <div className="flex items-center gap-2">
          <Book className="text-indigo-400" size={20} />
          <h3 className="text-white font-medium">Knowledge Base</h3>
        </div>
        {stats && stats.document_count > 0 && (
          <button
            onClick={() => setShowStats(!showStats)}
            className="text-zinc-400 hover:text-white transition-colors"
            title="Toggle stats"
          >
            <BarChart3 size={18} />
          </button>
        )}
      </div>

      {/* Stats Panel */}
      {showStats && stats && (
        <div className="mb-4 p-3 bg-black/50 border border-zinc-800 rounded-lg">
          <div className="grid grid-cols-3 gap-2 text-center">
            <div>
              <div className="text-lg font-semibold text-white">{stats.document_count}</div>
              <div className="text-xs text-zinc-500">Documents</div>
            </div>
            <div>
              <div className="text-lg font-semibold text-white">{stats.web_research_count}</div>
              <div className="text-xs text-zinc-500">Web Items</div>
            </div>
            <div>
              <div className="text-lg font-semibold text-indigo-400">~{stats.total_tokens.toLocaleString()}</div>
              <div className="text-xs text-zinc-500">Tokens</div>
            </div>
          </div>
          {Object.keys(stats.documents_by_type).length > 0 && (
            <div className="mt-3 pt-3 border-t border-zinc-800">
              <div className="text-xs text-zinc-500 mb-2">By Type:</div>
              <div className="flex flex-wrap gap-1">
                {Object.entries(stats.documents_by_type).map(([type, count]) => (
                  <span key={type} className={`text-xs px-2 py-0.5 rounded ${getDocTypeBadgeColor(type.toLowerCase())}`}>
                    {type}: {count}
                  </span>
                ))}
              </div>
            </div>
          )}
        </div>
      )}

      {/* Doc Type Selector */}
      <div className="mb-4">
        <label className="text-zinc-400 text-sm mb-2 block">Document Type</label>
        <select
          value={selectedDocType}
          onChange={(e) => setSelectedDocType(e.target.value)}
          className="w-full bg-black border border-zinc-700 rounded-lg px-4 py-2 text-white focus:border-indigo-500 focus:outline-none"
        >
          {DOC_TYPES.map((type) => (
            <option key={type.id} value={type.id}>{type.label}</option>
          ))}
        </select>
      </div>

      {/* Upload Zone */}
      <div
        onDragOver={handleDragOver}
        onDragLeave={handleDragLeave}
        onDrop={handleDrop}
        onClick={() => fileInputRef.current?.click()}
        className={`border-2 border-dashed rounded-lg p-6 text-center cursor-pointer transition-all mb-4 ${
          isDragging ? 'border-indigo-500 bg-indigo-500/10' : 'border-zinc-700 hover:border-zinc-500'
        }`}
      >
        <input ref={fileInputRef} type="file" multiple onChange={handleFileSelect} className="hidden" />
        {isLoading ? (
          <Loader2 size={24} className="mx-auto text-indigo-400 animate-spin" />
        ) : (
          <Upload size={24} className={`mx-auto mb-2 ${isDragging ? 'text-indigo-400' : 'text-zinc-500'}`} />
        )}
        <p className="text-sm text-zinc-400">Drag & drop or click to add documents</p>
      </div>

      {/* Documents List */}
      {documents.length > 0 && (
        <div className="mb-4">
          <div className="flex items-center justify-between mb-2">
            <span className="text-zinc-400 text-sm">{documents.length} document{documents.length !== 1 ? 's' : ''}</span>
            <button
              onClick={handlePreviewContext}
              disabled={isLoading}
              className="text-sm text-indigo-400 hover:text-indigo-300 flex items-center gap-1"
            >
              <Eye size={14} />
              Preview Context
            </button>
          </div>
          <div className="space-y-2 max-h-64 overflow-y-auto scrollbar-thin">
            {documents.map((doc) => (
              <div key={doc.id} className="bg-black border border-zinc-800 rounded-lg p-3 flex items-center justify-between group">
                <div className="flex items-center gap-3 flex-1 min-w-0">
                  {doc.doc_type === 'web_research' || doc.doc_type === 'WebResearch' ? (
                    <Globe size={16} className="text-cyan-400 flex-shrink-0" />
                  ) : (
                    <FileText size={16} className="text-indigo-400 flex-shrink-0" />
                  )}
                  <div className="flex-1 min-w-0">
                    <div className="flex items-center gap-2">
                      <p className="text-white text-sm truncate">{doc.name}</p>
                      {doc.auto_classified && (
                        <span title="Auto-classified">
                          <Sparkles size={12} className="text-amber-400 flex-shrink-0" />
                        </span>
                      )}
                    </div>
                    <div className="flex items-center gap-2">
                      <span className={`text-xs px-2 py-0.5 rounded ${getDocTypeBadgeColor(doc.doc_type.toLowerCase())}`}>
                        {DOC_TYPES.find(t => t.id === doc.doc_type.toLowerCase())?.label || doc.doc_type}
                      </span>
                      {doc.word_count && (
                        <span className="text-xs text-zinc-600">{doc.word_count.toLocaleString()} words</span>
                      )}
                    </div>
                  </div>
                </div>
                <button
                  onClick={() => handleRemoveDocument(doc.id)}
                  className="text-zinc-500 hover:text-red-400 opacity-0 group-hover:opacity-100 transition-opacity p-1"
                >
                  <Trash2 size={16} />
                </button>
              </div>
            ))}
          </div>
        </div>
      )}

      {/* Context Preview Modal */}
      {showPreview && (
        <div className="fixed inset-0 bg-black/80 flex items-center justify-center z-50 p-8">
          <div className="bg-zinc-900 border border-zinc-700 rounded-xl max-w-4xl w-full max-h-[80vh] flex flex-col">
            <div className="flex items-center justify-between p-4 border-b border-zinc-700">
              <h4 className="text-white font-medium">Compiled Context Preview</h4>
              <button onClick={() => setShowPreview(false)} className="text-zinc-400 hover:text-white">
                <X size={20} />
              </button>
            </div>
            <div className="p-4 overflow-y-auto flex-1 scrollbar-thin">
              <pre className="text-xs text-zinc-400 whitespace-pre-wrap font-mono">{previewContent}</pre>
            </div>
          </div>
        </div>
      )}

      {/* Empty State */}
      {documents.length === 0 && (
        <div className="text-center text-zinc-500 text-sm py-4">
          No documents added yet. Upload files or add web research to build your knowledge base.
        </div>
      )}
    </div>
  );
};

// Export addWebResearch function for integration with ResearchPanel
export { KnowledgePanel as default };
export type { KnowledgePanelProps };
