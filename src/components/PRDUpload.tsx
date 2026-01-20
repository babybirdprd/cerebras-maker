import { useState, useCallback, useRef, useEffect } from 'react';
import { Upload, FileText, X, FolderOpen, Sparkles, Layers } from 'lucide-react';
import { PRDFile } from '../types';
import { listTemplates, ProjectTemplate, openProjectDialog, createFromTemplate } from '../tauri-api';
import { useMakerStore } from '../store/makerStore';

interface PRDUploadProps {
  // No props needed after store migration
}

const PRDUpload: React.FC<PRDUploadProps> = () => {
  const { setPrdFile, setCurrentView, setAgentState, setWorkspacePath } = useMakerStore();
  const [isDragging, setIsDragging] = useState(false);
  const [uploadedFile, setUploadedFile] = useState<PRDFile | null>(null);
  const [templates, setTemplates] = useState<ProjectTemplate[]>([]);
  const fileInputRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    listTemplates().then(setTemplates).catch(() => {
      // Use mock templates if Tauri not available
      setTemplates([
        { id: 'tauri-react', name: 'Tauri + React', description: 'Desktop app with Tauri and React', tech_stack: ['Rust', 'TypeScript', 'React'] },
        { id: 'tauri-vanilla', name: 'Tauri + Vanilla JS', description: 'Lightweight desktop app', tech_stack: ['Rust', 'JavaScript'] },
        { id: 'rust-cli', name: 'Rust CLI', description: 'Command-line application', tech_stack: ['Rust'] },
      ]);
    });
  }, []);

  const handleDragOver = useCallback((e: React.DragEvent) => {
    e.preventDefault();
    setIsDragging(true);
  }, []);

  const handleDragLeave = useCallback((e: React.DragEvent) => {
    e.preventDefault();
    setIsDragging(false);
  }, []);

  const processFile = useCallback(async (file: File) => {
    const extension = file.name.split('.').pop()?.toLowerCase();
    if (!['md', 'txt', 'pdf'].includes(extension || '')) {
      alert('Please upload a .md, .txt, or .pdf file');
      return;
    }

    const content = await file.text();
    const prdFile: PRDFile = {
      name: file.name,
      content,
      type: extension as 'md' | 'txt' | 'pdf',
    };

    setUploadedFile(prdFile);
    setPrdFile(prdFile);
  }, [setPrdFile]);

  const handleDrop = useCallback((e: React.DragEvent) => {
    e.preventDefault();
    setIsDragging(false);

    const file = e.dataTransfer.files[0];
    if (file) {
      processFile(file);
    }
  }, [processFile]);

  const handleFileSelect = useCallback((e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0];
    if (file) {
      processFile(file);
    }
  }, [processFile]);

  const clearFile = () => {
    setUploadedFile(null);
    if (fileInputRef.current) {
      fileInputRef.current.value = '';
    }
  };

  return (
    <div className="h-full flex flex-col items-center justify-center p-8">
      <div className="max-w-2xl w-full space-y-8">
        {/* Header */}
        <div className="text-center">
          <div className="w-16 h-16 bg-linear-to-br from-indigo-500 to-purple-600 rounded-2xl flex items-center justify-center mx-auto mb-4 shadow-lg shadow-indigo-500/30">
            <Sparkles size={32} className="text-white" />
          </div>
          <h1 className="text-3xl font-bold text-white mb-2">Start Building</h1>
          <p className="text-zinc-400">Upload a PRD or open an existing project</p>
        </div>

        {/* Upload Zone */}
        {!uploadedFile ? (
          <div
            onDragOver={handleDragOver}
            onDragLeave={handleDragLeave}
            onDrop={handleDrop}
            onClick={() => fileInputRef.current?.click()}
            className={`border-2 border-dashed rounded-xl p-12 text-center cursor-pointer transition-all ${isDragging
              ? 'border-indigo-500 bg-indigo-500/10'
              : 'border-zinc-700 hover:border-zinc-500 hover:bg-zinc-900/50'
              }`}
          >
            <input
              ref={fileInputRef}
              type="file"
              accept=".md,.txt,.pdf"
              onChange={handleFileSelect}
              className="hidden"
            />
            <Upload size={48} className={`mx-auto mb-4 ${isDragging ? 'text-indigo-400' : 'text-zinc-500'}`} />
            <p className="text-lg text-white font-medium mb-2">
              {isDragging ? 'Drop your PRD here' : 'Drag & drop your PRD'}
            </p>
            <p className="text-sm text-zinc-500">
              or click to browse • Supports .md, .txt, .pdf
            </p>
          </div>
        ) : (
          <div className="bg-zinc-900 border border-zinc-700 rounded-xl p-6">
            <div className="flex items-center justify-between mb-4">
              <div className="flex items-center gap-3">
                <FileText size={24} className="text-indigo-400" />
                <div>
                  <p className="text-white font-medium">{uploadedFile.name}</p>
                  <p className="text-xs text-zinc-500">{uploadedFile.content.length} characters</p>
                </div>
              </div>
              <button onClick={clearFile} className="text-zinc-400 hover:text-white">
                <X size={20} />
              </button>
            </div>
            <button
              onClick={() => setCurrentView('interrogation')}
              className="mt-6 w-full py-3 bg-indigo-600 hover:bg-indigo-500 text-white rounded-xl font-medium transition-all shadow-lg shadow-indigo-500/20"
            >
              Analyze with L1 Orchestrator
            </button>
          </div>
        )}

        {/* Divider */}
        <div className="flex items-center gap-4">
          <div className="flex-1 h-px bg-zinc-800"></div>
          <span className="text-zinc-500 text-sm">or</span>
          <div className="flex-1 h-px bg-zinc-800"></div>
        </div>

        {/* Open Existing Project */}
        <button
          onClick={async () => {
            const path = await openProjectDialog();
            if (path) {
              setWorkspacePath(path);
              setCurrentView('topology');
            }
          }}
          className="w-full py-4 border border-zinc-700 hover:border-zinc-500 rounded-xl text-zinc-300 hover:text-white flex items-center justify-center gap-3 transition-all hover:bg-zinc-900/50"
        >
          <FolderOpen size={20} />
          Open Existing Project (Brownfield)
        </button>

        {/* Template Selection */}
        {templates.length > 0 && (
          <>
            <div className="flex items-center gap-4">
              <div className="flex-1 h-px bg-zinc-800"></div>
              <span className="text-zinc-500 text-sm">or start from template</span>
              <div className="flex-1 h-px bg-zinc-800"></div>
            </div>

            <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
              {templates.map((template) => (
                <button
                  key={template.id}
                  onClick={async () => {
                    const path = await openProjectDialog();
                    if (path) {
                      await createFromTemplate(template.id, path, template.name.toLowerCase().replace(/\s+/g, '-'));
                      setWorkspacePath(path);
                      setCurrentView('topology');
                    }
                  }}
                  className="p-4 border border-zinc-700 hover:border-indigo-500 rounded-xl text-left transition-all hover:bg-zinc-900/50 group"
                >
                  <div className="flex items-center gap-2 mb-2">
                    <Layers size={18} className="text-indigo-400" />
                    <span className="text-white font-medium">{template.name}</span>
                  </div>
                  <p className="text-xs text-zinc-500 mb-3">{template.description}</p>
                  <div className="flex flex-wrap gap-1">
                    {template.tech_stack.map((tech) => (
                      <span key={tech} className="px-2 py-0.5 bg-zinc-800 text-zinc-400 text-xs rounded">
                        {tech}
                      </span>
                    ))}
                  </div>
                </button>
              ))}
            </div>
          </>
        )}
      </div>
    </div>
  );
};

export default PRDUpload;

