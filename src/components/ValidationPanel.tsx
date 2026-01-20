// ValidationPanel.tsx - Multi-file Edit Validation UI
// PRD Section 3.2: Architectural Red-Flagging with Grits VirtualApply

import { useState } from 'react';
import {
  Shield,
  CheckCircle2,
  XCircle,
  AlertTriangle,
  Loader2,
  Plus,
  Trash2,
  FileCode,
  GitBranch,
  Layers
} from 'lucide-react';
import {
  validateMultiFileEdit,
  previewEditImpact,
  MultiFileEdit,
  MultiFileValidationResult,
  EditImpactPreview
} from '../tauri-api';
import { useMakerStore } from '../store/makerStore';

interface ValidationPanelProps {
  className?: string; // Optional if needed
}

export const ValidationPanel: React.FC<ValidationPanelProps> = () => {
  const { workspacePath } = useMakerStore();
  const [edits, setEdits] = useState<MultiFileEdit[]>([
    { file_path: '', operation: 'modify', content: '' }
  ]);
  const [validationResult, setValidationResult] = useState<MultiFileValidationResult | null>(null);
  const [impactPreview, setImpactPreview] = useState<EditImpactPreview | null>(null);
  const [isValidating, setIsValidating] = useState(false);
  const [isPreviewing, setIsPreviewing] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const addEdit = () => {
    setEdits([...edits, { file_path: '', operation: 'modify', content: '' }]);
  };

  const removeEdit = (index: number) => {
    if (edits.length > 1) {
      setEdits(edits.filter((_, i) => i !== index));
    }
  };

  const updateEdit = (index: number, field: keyof MultiFileEdit, value: string) => {
    const newEdits = [...edits];
    (newEdits[index] as any)[field] = value;
    setEdits(newEdits);
  };

  const handleValidate = async () => {
    if (!workspacePath || edits.every(e => !e.file_path)) return;
    setIsValidating(true);
    setError(null);
    setValidationResult(null);
    try {
      const result = await validateMultiFileEdit(workspacePath, edits.filter(e => e.file_path));
      setValidationResult(result);
    } catch (e) {
      setError(String(e));
    } finally {
      setIsValidating(false);
    }
  };

  const handlePreview = async () => {
    if (edits.every(e => !e.file_path)) return;
    setIsPreviewing(true);
    setError(null);
    setImpactPreview(null);
    try {
      const preview = await previewEditImpact(edits.filter(e => e.file_path));
      setImpactPreview(preview);
    } catch (e) {
      setError(String(e));
    } finally {
      setIsPreviewing(false);
    }
  };

  return (
    <div className="p-4 h-full overflow-y-auto bg-zinc-900">
      <div className="flex items-center justify-between mb-4">
        <h3 className="text-lg font-semibold text-white flex items-center gap-2">
          <Shield size={20} className="text-indigo-400" />
          Edit Validation
        </h3>
      </div>

      <p className="text-xs text-zinc-400 mb-4">
        Validate proposed file changes against architectural rules before applying them.
        Uses Grits VirtualApply to detect cycles and layer violations.
      </p>

      {/* Edits List */}
      <div className="space-y-3 mb-4">
        {edits.map((edit, index) => (
          <div key={index} className="bg-black border border-zinc-800 rounded-lg p-3">
            <div className="flex items-center justify-between mb-2">
              <span className="text-xs text-zinc-500">File #{index + 1}</span>
              {edits.length > 1 && (
                <button
                  onClick={() => removeEdit(index)}
                  className="p-1 text-zinc-500 hover:text-red-400"
                >
                  <Trash2 size={14} />
                </button>
              )}
            </div>
            <input
              type="text"
              value={edit.file_path}
              onChange={(e) => updateEdit(index, 'file_path', e.target.value)}
              placeholder="src/components/MyFile.tsx"
              className="w-full bg-zinc-800 border border-zinc-700 rounded px-2 py-1.5 text-white text-sm mb-2"
            />
            <div className="flex gap-2 mb-2">
              {(['create', 'modify', 'delete'] as const).map((op) => (
                <button
                  key={op}
                  onClick={() => updateEdit(index, 'operation', op)}
                  className={`px-2 py-1 text-xs rounded ${edit.operation === op
                    ? 'bg-indigo-600 text-white'
                    : 'bg-zinc-800 text-zinc-400 hover:text-white'
                    }`}
                >
                  {op}
                </button>
              ))}
            </div>
            {edit.operation !== 'delete' && (
              <textarea
                value={edit.content || ''}
                onChange={(e) => updateEdit(index, 'content', e.target.value)}
                placeholder="Paste code content here..."
                className="w-full bg-zinc-800 border border-zinc-700 rounded px-2 py-1.5 text-white text-xs h-20 resize-none font-mono"
              />
            )}
          </div>
        ))}
      </div>

      {/* Add Edit Button */}
      <button
        onClick={addEdit}
        className="w-full py-2 mb-4 border border-dashed border-zinc-700 text-zinc-400 hover:text-white hover:border-zinc-500 rounded-lg flex items-center justify-center gap-2 text-sm"
      >
        <Plus size={16} /> Add Another File
      </button>

      {/* Action Buttons */}
      <div className="flex gap-2 mb-4">
        <button
          onClick={handlePreview}
          disabled={isPreviewing || edits.every(e => !e.file_path)}
          className="flex-1 py-2 bg-zinc-700 hover:bg-zinc-600 disabled:bg-zinc-800 text-white rounded-lg font-medium flex items-center justify-center gap-2 text-sm"
        >
          {isPreviewing ? <Loader2 size={14} className="animate-spin" /> : <FileCode size={14} />}
          Preview Impact
        </button>
        <button
          onClick={handleValidate}
          disabled={isValidating || edits.every(e => !e.file_path)}
          className="flex-1 py-2 bg-indigo-600 hover:bg-indigo-500 disabled:bg-zinc-700 text-white rounded-lg font-medium flex items-center justify-center gap-2 text-sm"
        >
          {isValidating ? <Loader2 size={14} className="animate-spin" /> : <Shield size={14} />}
          Validate
        </button>
      </div>

      {/* Impact Preview */}
      {impactPreview && (
        <div className="mb-4 bg-zinc-800/50 border border-zinc-700 rounded-lg p-3">
          <h4 className="text-sm font-medium text-zinc-300 mb-2 flex items-center gap-2">
            <FileCode size={14} /> Impact Preview
          </h4>
          <div className="text-xs space-y-2">
            <div className="flex justify-between">
              <span className="text-zinc-400">Files affected:</span>
              <span className="text-white">{impactPreview.files_affected}</span>
            </div>
            <div className="flex justify-between">
              <span className="text-zinc-400">New symbols:</span>
              <span className="text-white">{impactPreview.new_symbols.length}</span>
            </div>
            <div className="flex justify-between">
              <span className="text-zinc-400">New dependencies:</span>
              <span className="text-white">{impactPreview.new_dependencies.length}</span>
            </div>
            {impactPreview.new_symbols.length > 0 && (
              <div className="mt-2 pt-2 border-t border-zinc-700">
                <span className="text-zinc-400">Symbols: </span>
                <span className="text-indigo-300">{impactPreview.new_symbols.slice(0, 5).join(', ')}</span>
                {impactPreview.new_symbols.length > 5 && <span className="text-zinc-500"> +{impactPreview.new_symbols.length - 5} more</span>}
              </div>
            )}
          </div>
        </div>
      )}

      {/* Validation Result */}
      {validationResult && (
        <div className={`p-4 rounded-lg border ${validationResult.is_safe
          ? 'bg-green-500/10 border-green-500/30'
          : 'bg-red-500/10 border-red-500/30'
          }`}>
          <div className="flex items-center gap-2 mb-3">
            {validationResult.is_safe ? (
              <>
                <CheckCircle2 size={20} className="text-green-400" />
                <span className="font-medium text-green-300">Changes are Safe</span>
              </>
            ) : (
              <>
                <XCircle size={20} className="text-red-400" />
                <span className="font-medium text-red-300">Architectural Issues Detected</span>
              </>
            )}
          </div>

          {/* Metrics */}
          <div className="grid grid-cols-3 gap-2 mb-3 text-center text-xs">
            <div className="bg-black/30 rounded p-2">
              <div className="text-sm font-medium text-white">{validationResult.files_analyzed}</div>
              <div className="text-zinc-500">Files</div>
            </div>
            <div className="bg-black/30 rounded p-2">
              <div className="text-sm font-medium text-white">{validationResult.new_symbols.length}</div>
              <div className="text-zinc-500">Symbols</div>
            </div>
            <div className="bg-black/30 rounded p-2">
              <div className={`text-sm font-medium ${validationResult.introduces_cycles ? 'text-red-400' : 'text-white'}`}>
                {validationResult.new_betti_1 - validationResult.original_betti_1}
              </div>
              <div className="text-zinc-500">New Cycles</div>
            </div>
          </div>

          {/* Cycle Warning */}
          {validationResult.introduces_cycles && (
            <div className="flex items-start gap-2 mb-2 p-2 bg-red-500/20 rounded">
              <GitBranch size={14} className="text-red-400 mt-0.5" />
              <div className="text-xs text-red-300">
                Would introduce {validationResult.new_betti_1 - validationResult.original_betti_1} new cycle(s).
                Betti_1: {validationResult.original_betti_1} → {validationResult.new_betti_1}
              </div>
            </div>
          )}

          {/* Layer Violations */}
          {validationResult.layer_violations.length > 0 && (
            <div className="mb-2">
              <div className="flex items-center gap-1 text-xs text-red-300 mb-1">
                <Layers size={12} /> Layer Violations ({validationResult.layer_violations.length})
              </div>
              {validationResult.layer_violations.map((v, i) => (
                <div key={i} className="text-xs text-red-200 ml-4 mb-1">
                  • {v.from_symbol} ({v.from_layer}) → {v.to_symbol} ({v.to_layer})
                </div>
              ))}
            </div>
          )}

          {/* Cross-file Issues */}
          {validationResult.cross_file_issues.length > 0 && (
            <div className="mb-2">
              <div className="flex items-center gap-1 text-xs text-yellow-300 mb-1">
                <AlertTriangle size={12} /> Cross-file Issues ({validationResult.cross_file_issues.length})
              </div>
              {validationResult.cross_file_issues.map((issue, i) => (
                <div key={i} className="text-xs text-yellow-200 ml-4 mb-1">• {issue}</div>
              ))}
            </div>
          )}

          {/* Errors */}
          {validationResult.errors.length > 0 && (
            <div className="mt-2 pt-2 border-t border-red-500/30">
              <div className="text-xs text-red-300 font-medium mb-1">Errors:</div>
              {validationResult.errors.map((err, i) => (
                <div key={i} className="text-xs text-red-200">• {err}</div>
              ))}
            </div>
          )}
        </div>
      )}

      {/* Error Display */}
      {error && (
        <div className="mt-4 p-3 bg-red-500/10 border border-red-500/30 rounded-lg flex items-start gap-2">
          <AlertTriangle size={16} className="text-red-400 mt-0.5 shrink-0" />
          <span className="text-red-300 text-sm">{error}</span>
        </div>
      )}
    </div>
  );
};

export default ValidationPanel;
