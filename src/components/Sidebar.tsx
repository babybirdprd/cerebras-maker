import React from 'react';
import { LayoutDashboard, Network, History, Settings, Terminal, ShieldCheck } from 'lucide-react';

interface SidebarProps {
  currentView: string;
  onChangeView: (view: string) => void;
  onOpenSettings: () => void;
  className?: string;
}

const navItems = [
  { id: 'dashboard', label: 'Blueprint', icon: LayoutDashboard },
  { id: 'topology', label: 'Topology', icon: Network },
  { id: 'execution', label: 'Swarm', icon: Terminal },
  { id: 'history', label: 'Shadow Git', icon: History },
];

export const Sidebar: React.FC<SidebarProps> = ({ currentView, onChangeView, onOpenSettings, className = '' }) => {
  return (
    <div className={`w-64 h-full bg-zinc-950 border-r border-zinc-800 flex flex-col justify-between ${className}`}>
      <div>
        <div className="p-6 flex items-center gap-3">
          <div className="w-8 h-8 bg-gradient-to-br from-indigo-500 to-purple-600 rounded flex items-center justify-center font-bold text-white shadow-lg shadow-indigo-500/20">
            C
          </div>
          <div>
            <h1 className="font-bold text-lg tracking-tight text-white leading-none">MAKER</h1>
            <span className="text-xs text-zinc-500 font-mono">v5.0.0-rc1</span>
          </div>
        </div>

        <nav className="mt-6 px-3 space-y-1">
          {navItems.map((item) => (
            <button
              key={item.id}
              onClick={() => onChangeView(item.id)}
              className={`w-full flex items-center gap-3 px-3 py-2.5 rounded-md text-sm transition-all ${
                currentView === item.id
                  ? 'bg-zinc-800 text-white font-medium border border-zinc-700'
                  : 'text-zinc-400 hover:text-white hover:bg-zinc-900'
              }`}
            >
              <item.icon size={18} />
              {item.label}
            </button>
          ))}
        </nav>
      </div>

      <div className="p-4 border-t border-zinc-800">
        <div className="bg-zinc-900 rounded-lg p-3 border border-zinc-800 mb-2">
            <div className="flex items-center justify-between mb-2">
                <span className="text-xs text-zinc-400 font-mono uppercase">Reliability</span>
                <ShieldCheck size={14} className="text-emerald-500" />
            </div>
            <div className="w-full bg-zinc-800 h-1.5 rounded-full overflow-hidden">
                <div className="bg-emerald-500 w-[99.9%] h-full"></div>
            </div>
            <div className="flex justify-between mt-1">
                <span className="text-[10px] text-zinc-500">Grits Core</span>
                <span className="text-[10px] text-emerald-400 font-mono">100% Solid</span>
            </div>
        </div>
        <button 
          onClick={onOpenSettings}
          className="flex items-center gap-3 px-3 py-2 text-zinc-400 hover:text-white w-full text-sm"
        >
          <Settings size={18} />
          Settings
        </button>
      </div>
    </div>
  );
};

export const MobileNav: React.FC<SidebarProps> = ({ currentView, onChangeView, onOpenSettings, className = '' }) => {
  return (
    <div className={`h-16 bg-zinc-950 border-t border-zinc-800 flex items-center justify-around px-2 ${className}`}>
      {navItems.map((item) => (
        <button
          key={item.id}
          onClick={() => onChangeView(item.id)}
          className={`flex flex-col items-center justify-center p-2 rounded-md transition-all ${
            currentView === item.id
              ? 'text-indigo-400'
              : 'text-zinc-500 hover:text-zinc-300'
          }`}
        >
          <item.icon size={20} />
          <span className="text-[10px] mt-1 font-medium">{item.label}</span>
        </button>
      ))}
      <button 
        onClick={onOpenSettings}
        className="flex flex-col items-center justify-center p-2 text-zinc-500 hover:text-zinc-300"
      >
          <Settings size={20} />
          <span className="text-[10px] mt-1 font-medium">Config</span>
      </button>
    </div>
  );
};

export default Sidebar;

