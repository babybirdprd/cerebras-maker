import React, { useState } from 'react';
import { Circle, CheckCircle2, CircleDashed, ChevronRight, ChevronDown, FileText, AlertTriangle, Code, StickyNote, Hash } from 'lucide-react';
import { MOCK_PLAN } from '../constants';
import { Task } from '../types';

const StatusIcon = ({ status }: { status: Task['status'] }) => {
  switch (status) {
    case 'completed': return <CheckCircle2 size={16} className="text-emerald-500" />;
    case 'active': return <CircleDashed size={16} className="text-indigo-500 animate-spin-slow" />;
    case 'pending': return <Circle size={16} className="text-zinc-600" />;
    case 'failed': return <AlertTriangle size={16} className="text-rose-500" />;
    default: return <Circle size={16} className="text-zinc-600" />;
  }
};

const TaskItem: React.FC<{ task: Task }> = ({ task }) => {
  const [childrenExpanded, setChildrenExpanded] = useState(true);
  const [detailsExpanded, setDetailsExpanded] = useState(false);
  const hasChildren = task.children && task.children.length > 0;
  const hasDetails = task.details && (task.details.issues.length > 0 || task.details.snippet || task.details.notes);

  const toggleChildren = (e: React.MouseEvent) => {
    e.stopPropagation();
    setChildrenExpanded(!childrenExpanded);
  };

  const toggleDetails = () => {
    if (hasDetails) {
        setDetailsExpanded(!detailsExpanded);
    }
  };

  const containerClasses = task.status === 'active'
    ? 'bg-gradient-to-r from-indigo-500/20 to-transparent border-l-2 border-indigo-500 shadow-[inset_0_0_20px_rgba(99,102,241,0.05)]'
    : 'border-l-2 border-transparent hover:bg-zinc-800/50';

  const textClasses = task.status === 'active' 
    ? 'text-white font-semibold drop-shadow-[0_0_5px_rgba(255,255,255,0.3)]' 
    : 'text-zinc-400 group-hover:text-zinc-200';

  return (
    <div className="select-none mb-1">
      <div 
        className={`relative flex items-center gap-2 py-2.5 px-2 rounded-r cursor-pointer group transition-all duration-200 ${containerClasses} ${detailsExpanded ? 'bg-zinc-800/30' : ''}`}
        style={{ paddingLeft: `${task.depth * 16 + 8}px` }}
        onClick={toggleDetails}
      >
        <div 
            className={`flex-shrink-0 w-4 h-4 flex items-center justify-center text-zinc-500 hover:text-white transition-colors ${!hasChildren ? 'invisible' : ''}`}
            onClick={toggleChildren}
        >
             {childrenExpanded ? <ChevronDown size={14} /> : <ChevronRight size={14} />}
        </div>
        
        <StatusIcon status={task.status} />
        
        <span className={`text-sm font-mono truncate ${textClasses}`}>
          {task.title}
        </span>
        
        {task.status === 'active' && (
             <span className="ml-auto text-[10px] bg-indigo-600 text-white px-2 py-0.5 rounded font-bold uppercase tracking-wider hidden sm:inline-block shadow-lg shadow-indigo-500/20 animate-pulse">
                 Active
             </span>
        )}
      </div>

      {detailsExpanded && hasDetails && (
          <div 
            className="my-1 mr-2 bg-black/40 rounded border border-zinc-800/60 p-3 text-xs flex flex-col gap-3 shadow-inner"
            style={{ marginLeft: `${task.depth * 16 + 32}px` }}
          >
              {task.details?.notes && (
                  <div className="flex gap-2">
                      <StickyNote size={14} className="text-zinc-500 shrink-0 mt-0.5" />
                      <p className="text-zinc-300 italic">{task.details.notes}</p>
                  </div>
              )}
              {task.details?.issues && task.details.issues.length > 0 && (
                  <div className="flex gap-2">
                      <Hash size={14} className="text-zinc-500 shrink-0 mt-0.5" />
                      <div className="flex flex-col gap-1">
                          {task.details.issues.map((issue, idx) => (
                              <span key={idx} className="text-indigo-400 hover:text-indigo-300 underline cursor-pointer">{issue}</span>
                          ))}
                      </div>
                  </div>
              )}
              {task.details?.snippet && (
                  <div className="flex flex-col gap-2 mt-1">
                      <div className="flex items-center gap-2 text-zinc-500">
                          <Code size={14} />
                          <span className="text-[10px] uppercase font-bold tracking-wider">Relevant Context</span>
                      </div>
                      <div className="bg-zinc-950 p-2 rounded border border-zinc-800 font-mono text-zinc-400 overflow-x-auto">
                          <pre>{task.details.snippet}</pre>
                      </div>
                  </div>
              )}
          </div>
      )}

      {childrenExpanded && hasChildren && (
        <div className="mt-0.5">
          {task.children!.map(child => (
            <TaskItem key={child.id} task={child} />
          ))}
        </div>
      )}
    </div>
  );
};

const PlanView: React.FC = () => {
  return (
    <div className="h-full flex flex-col p-4 lg:p-6">
       <div className="mb-4 lg:mb-6 flex justify-between items-end">
           <div>
               <h2 className="text-xl font-bold text-white">Execution Plan</h2>
               <p className="text-zinc-400 text-sm mt-1">System 2 Decomposition • Recursive Depth: 3</p>
           </div>
           <button className="flex items-center gap-2 text-xs bg-zinc-800 hover:bg-zinc-700 text-zinc-300 px-3 py-1.5 rounded border border-zinc-700 transition-colors">
               <FileText size={14} />
               <span className="hidden sm:inline">View Source</span>
           </button>
       </div>
       
       <div className="flex-1 bg-zinc-900 border border-zinc-800 rounded-lg overflow-y-auto p-2 scrollbar-thin">
           {MOCK_PLAN.map(task => (
               <TaskItem key={task.id} task={task} />
           ))}
       </div>
    </div>
  );
};

export default PlanView;

