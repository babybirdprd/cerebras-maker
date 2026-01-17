import { useState } from 'react';
import { Globe, Search, FileText, Loader2, ExternalLink, AlertCircle, CheckCircle2, X } from 'lucide-react';
import { crawlUrl, researchDocs, CrawlResult, ResearchResult } from '../hooks/useTauri';

interface ResearchPanelProps {
  onResearchComplete?: (content: string) => void;
  className?: string;
}

const ResearchPanel: React.FC<ResearchPanelProps> = ({ onResearchComplete, className = '' }) => {
  const [urls, setUrls] = useState<string[]>(['']);
  const [isLoading, setIsLoading] = useState(false);
  const [results, setResults] = useState<ResearchResult | null>(null);
  const [singleResult, setSingleResult] = useState<CrawlResult | null>(null);
  const [error, setError] = useState<string | null>(null);

  const addUrlField = () => setUrls([...urls, '']);
  const removeUrlField = (index: number) => {
    if (urls.length > 1) setUrls(urls.filter((_, i) => i !== index));
  };
  const updateUrl = (index: number, value: string) => {
    const newUrls = [...urls];
    newUrls[index] = value;
    setUrls(newUrls);
  };

  const handleSingleCrawl = async (url: string) => {
    if (!url.trim()) return;
    setIsLoading(true);
    setError(null);
    setSingleResult(null);
    setResults(null);
    try {
      const result = await crawlUrl(url.trim(), true);
      setSingleResult(result);
      if (result.markdown && onResearchComplete) {
        onResearchComplete(result.markdown);
      }
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Failed to crawl URL');
    } finally {
      setIsLoading(false);
    }
  };

  const handleResearch = async () => {
    const validUrls = urls.filter(u => u.trim());
    if (validUrls.length === 0) return;
    if (validUrls.length === 1) {
      return handleSingleCrawl(validUrls[0]);
    }
    setIsLoading(true);
    setError(null);
    setResults(null);
    setSingleResult(null);
    try {
      const res = await researchDocs(validUrls);
      setResults(res);
      if (res.documents.length > 0 && onResearchComplete) {
        const combinedMarkdown = res.documents
          .map(d => `# ${d.title || d.url}\n\n${d.markdown || '(No content)'}`)
          .join('\n\n---\n\n');
        onResearchComplete(combinedMarkdown);
      }
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Research failed');
    } finally {
      setIsLoading(false);
    }
  };

  return (
    <div className={`bg-zinc-900 border border-zinc-700 rounded-xl p-6 ${className}`}>
      <div className="flex items-center gap-2 mb-4">
        <Globe className="text-indigo-400" size={20} />
        <h3 className="text-white font-medium">Research Documentation</h3>
      </div>
      <p className="text-zinc-400 text-sm mb-4">
        Crawl documentation, API references, or any web page to add to your project context.
      </p>

      {/* URL Input Fields */}
      <div className="space-y-2 mb-4">
        {urls.map((url, index) => (
          <div key={index} className="flex gap-2">
            <input
              type="url"
              value={url}
              onChange={(e) => updateUrl(index, e.target.value)}
              placeholder="https://docs.example.com/api"
              className="flex-1 bg-black border border-zinc-700 rounded-lg px-4 py-2 text-white placeholder-zinc-500 focus:border-indigo-500 focus:outline-none"
            />
            {urls.length > 1 && (
              <button onClick={() => removeUrlField(index)} className="text-zinc-500 hover:text-white p-2">
                <X size={18} />
              </button>
            )}
          </div>
        ))}
      </div>

      {/* Action Buttons */}
      <div className="flex gap-2 mb-4">
        <button onClick={addUrlField} className="px-4 py-2 text-sm text-zinc-400 hover:text-white border border-zinc-700 hover:border-zinc-500 rounded-lg transition-colors">
          + Add URL
        </button>
        <button
          onClick={handleResearch}
          disabled={isLoading || urls.every(u => !u.trim())}
          className="flex-1 py-2 bg-indigo-600 hover:bg-indigo-500 disabled:bg-zinc-700 disabled:cursor-not-allowed text-white rounded-lg font-medium flex items-center justify-center gap-2"
        >
          {isLoading ? <Loader2 size={18} className="animate-spin" /> : <Search size={18} />}
          {isLoading ? 'Crawling...' : 'Research'}
        </button>
      </div>

      {/* Error Display */}
      {error && (
        <div className="p-3 bg-red-500/10 border border-red-500/30 rounded-lg flex items-start gap-2 mb-4">
          <AlertCircle size={18} className="text-red-400 mt-0.5" />
          <span className="text-red-300 text-sm">{error}</span>
        </div>
      )}

      {/* Single Result Display */}
      {singleResult && (
        <div className="bg-black border border-zinc-800 rounded-lg p-4">
          <div className="flex items-center justify-between mb-2">
            <div className="flex items-center gap-2">
              <CheckCircle2 size={16} className="text-green-400" />
              <span className="text-white text-sm font-medium">{singleResult.title || 'Untitled'}</span>
            </div>
            <a href={singleResult.url} target="_blank" rel="noopener noreferrer" className="text-zinc-500 hover:text-indigo-400">
              <ExternalLink size={14} />
            </a>
          </div>
          <p className="text-zinc-500 text-xs mb-2">Crawled in {singleResult.duration_ms}ms • Status: {singleResult.status_code}</p>
          <div className="max-h-48 overflow-y-auto scrollbar-thin">
            <pre className="text-xs text-zinc-400 whitespace-pre-wrap">{singleResult.markdown?.slice(0, 1000)}...</pre>
          </div>
        </div>
      )}

      {/* Multi-Result Display */}
      {results && (
        <div className="space-y-2">
          <div className="flex items-center justify-between text-sm mb-2">
            <span className="text-green-400">{results.success_count} succeeded</span>
            {results.error_count > 0 && <span className="text-red-400">{results.error_count} failed</span>}
          </div>
          {results.documents.map((doc, i) => (
            <div key={i} className="bg-black border border-zinc-800 rounded-lg p-3">
              <div className="flex items-center gap-2">
                <FileText size={14} className="text-indigo-400" />
                <span className="text-white text-sm truncate flex-1">{doc.title || doc.url}</span>
                <a href={doc.url} target="_blank" rel="noopener noreferrer" className="text-zinc-500 hover:text-indigo-400">
                  <ExternalLink size={14} />
                </a>
              </div>
            </div>
          ))}
        </div>
      )}
    </div>
  );
};

export default ResearchPanel;

