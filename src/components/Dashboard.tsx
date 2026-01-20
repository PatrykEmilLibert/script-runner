import { motion } from "framer-motion";
import { BarChart3, Zap, GitBranch } from "lucide-react";

interface DashboardProps {
  scripts: string[];
}

export default function Dashboard({ scripts }: DashboardProps) {
  return (
    <motion.div className="dashboard" initial={{ opacity: 0 }} animate={{ opacity: 1 }}>
      <div className="dashboard-grid">
        <motion.div className="card" whileHover={{ scale: 1.05 }}>
          <BarChart3 className="card-icon" />
          <h3>Total Scripts</h3>
          <p className="card-value">{scripts.length}</p>
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
        <h2>Available Scripts</h2>
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
