import { motion } from "framer-motion";
import { BarChart3, Zap, GitBranch, Plus } from "lucide-react";

interface DashboardProps {
  scripts: string[];
  onAddScript: () => void;
  isAdmin: boolean;
  officialScripts: string[];
}

export default function Dashboard({ scripts, onAddScript, isAdmin, officialScripts }: DashboardProps) {
  return (
    <motion.div className="dashboard" initial={{ opacity: 0 }} animate={{ opacity: 1 }}>
      <div className="dashboard-grid">
        <motion.div className="card" whileHover={{ scale: 1.05 }}>
          <BarChart3 className="card-icon" />
          <h3>Total Scripts</h3>
          <p className="card-value">{scripts.length + officialScripts.length}</p>
        </motion.div>

        <motion.div className="card" whileHover={{ scale: 1.05 }}>
          <Zap className="card-icon" />
          <h3>Status</h3>
          <p className="card-value">Active</p>
        </motion.div>

        <motion.div className="card" whileHover={{ scale: 1.05 }}>
          <GitBranch className="card-icon" />
          <h3>Last Sync</h3>
          <p className="card-value">Now</p>
        </motion.div>
      </div>

      <motion.div className="recent-scripts" initial={{ y: 20, opacity: 0 }} animate={{ y: 0, opacity: 1 }} transition={{ delay: 0.2 }}>
        <div className="flex justify-between items-center mb-4">
          <h2>Official Scripts</h2>
          {isAdmin && (
            <button
              onClick={onAddScript}
              className="flex items-center gap-2 bg-gradient-to-r from-blue-600 to-purple-600 hover:from-blue-700 hover:to-purple-700 px-4 py-2 rounded-lg font-medium transition-all"
            >
              <Plus size={20} />
              Add Script
            </button>
          )}
        </div>
        <ul>
          {officialScripts.slice(0, 5).map((script) => (
            <li key={script}>
              <span className="script-icon">🔒</span>
              {script}
            </li>
          ))}
        </ul>

        <h3 className="mt-6 mb-2">User Scripts</h3>
        <ul>
          {scripts.slice(0, 5).map((script) => (
            <li key={script}>
              <span className="script-icon">📄</span>
              {script}
            </li>
          ))}
        </ul>
      </motion.div>
    </motion.div>
  );
}
