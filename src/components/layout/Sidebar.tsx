import { NavLink } from 'react-router-dom';
import { Terminal, Activity } from 'lucide-react';

const navItems = [
  { to: '/claude-config', icon: Terminal, label: 'Claude Config' },
  { to: '/zenmux-quota', icon: Activity, label: 'ZenMux Quota' },
];

export function Sidebar() {
  return (
    <aside className="flex h-screen w-56 flex-col border-r border-border bg-sidebar">
      <div className="p-4">
        <h1 className="text-sm font-semibold text-sidebar-foreground">Toolkit</h1>
      </div>
      <nav className="flex-1 space-y-1 px-2">
        {navItems.map((item) => (
          <NavLink
            key={item.to}
            to={item.to}
            className={({ isActive }) =>
              `flex items-center gap-2 rounded-md px-3 py-2 text-sm transition-colors ${
                isActive
                  ? 'bg-sidebar-accent text-sidebar-accent-foreground'
                  : 'text-sidebar-foreground/70 hover:bg-sidebar-accent/50 hover:text-sidebar-accent-foreground'
              }`
            }
          >
            <item.icon className="h-4 w-4" />
            {item.label}
          </NavLink>
        ))}
      </nav>
    </aside>
  );
}
