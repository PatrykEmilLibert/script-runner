import { motion } from "framer-motion";

interface ScriptListProps {
  scripts: string[];
  selected: string | null;
  onSelect: (script: string) => void;
}

export default function ScriptList({ scripts, selected, onSelect }: ScriptListProps) {
  return (
    <motion.div className="script-list" initial={{ x: -20, opacity: 0 }} animate={{ x: 0, opacity: 1 }}>
      <h2>Scripts</h2>
      <div className="scripts-container">
        {scripts.map((script) => (
          <motion.button
            key={script}
            className={`script-item ${selected === script ? "active" : ""}`}
            onClick={() => onSelect(script)}
            whileHover={{ scale: 1.02 }}
            whileTap={{ scale: 0.98 }}
          >
            <span className="script-name">{script}</span>
            <span className="script-status">▶</span>
          </motion.button>
        ))}
      </div>
    </motion.div>
  );
}
