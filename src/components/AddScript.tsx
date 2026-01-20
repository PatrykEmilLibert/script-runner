import React, { useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { motion } from 'framer-motion';

interface AddScriptProps {
  onScriptAdded: () => void;
  onClose: () => void;
  scriptsDir: string;
}

export const AddScript: React.FC<AddScriptProps> = ({ onScriptAdded, onClose, scriptsDir }) => {
  const [scriptName, setScriptName] = useState('');
  const [description, setDescription] = useState('');
  const [author, setAuthor] = useState('');
  const [scriptContent, setScriptContent] = useState('');
  const [isSubmitting, setIsSubmitting] = useState(false);
  const [error, setError] = useState('');
  const [detectedDeps, setDetectedDeps] = useState<string[]>([]);

  const analyzeCode = (code: string) => {
    const deps = new Set<string>();
    const lines = code.split('\n');
    
    const stdlibModules = new Set([
      'os', 'sys', 're', 'json', 'time', 'datetime', 'random', 'math',
      'collections', 'itertools', 'functools', 'pathlib', 'typing',
      'logging', 'argparse', 'subprocess', 'threading', 'multiprocessing'
    ]);
    
    for (const line of lines) {
      const trimmed = line.trim();
      
      // Match: import module
      const importMatch = trimmed.match(/^import\s+(\w+)/);
      if (importMatch && !stdlibModules.has(importMatch[1])) {
        deps.add(importMatch[1]);
      }
      
      // Match: from module import ...
      const fromMatch = trimmed.match(/^from\s+(\w+)/);
      if (fromMatch && !stdlibModules.has(fromMatch[1])) {
        deps.add(fromMatch[1]);
      }
    }
    
    setDetectedDeps(Array.from(deps).sort());
  };

  const handleCodeChange = (code: string) => {
    setScriptContent(code);
    analyzeCode(code);
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    
    if (!scriptName || !scriptContent) {
      setError('Script name and content are required');
      return;
    }
    
    setIsSubmitting(true);
    setError('');
    
    try {
      const result = await invoke<string>('add_script', {
        scriptName: scriptName.trim(),
        scriptContent,
        description: description || 'No description provided',
        author: author || 'Unknown',
        scriptsDir,
      });
      
      console.log(result);
      onScriptAdded();
      onClose();
    } catch (err) {
      setError(err as string);
    } finally {
      setIsSubmitting(false);
    }
  };

  return (
    <motion.div
      initial={{ opacity: 0 }}
      animate={{ opacity: 1 }}
      exit={{ opacity: 0 }}
      className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50 p-4"
      onClick={onClose}
    >
      <motion.div
        initial={{ scale: 0.9, opacity: 0 }}
        animate={{ scale: 1, opacity: 1 }}
        exit={{ scale: 0.9, opacity: 0 }}
        onClick={(e) => e.stopPropagation()}
        className="bg-gray-800 rounded-lg shadow-2xl max-w-4xl w-full max-h-[90vh] overflow-y-auto"
      >
        <div className="p-6 border-b border-gray-700 flex justify-between items-center sticky top-0 bg-gray-800 z-10">
          <h2 className="text-2xl font-bold">✨ Add New Script</h2>
          <button
            onClick={onClose}
            className="text-gray-400 hover:text-white text-2xl"
          >
            ×
          </button>
        </div>

        <form onSubmit={handleSubmit} className="p-6 space-y-4">
          {error && (
            <div className="bg-red-900 border border-red-700 text-red-200 px-4 py-3 rounded">
              {error}
            </div>
          )}

          <div>
            <label className="block text-sm font-medium mb-2">
              Script Name *
            </label>
            <input
              type="text"
              value={scriptName}
              onChange={(e) => setScriptName(e.target.value)}
              className="w-full bg-gray-700 border border-gray-600 rounded px-3 py-2 focus:outline-none focus:border-blue-500"
              placeholder="my_awesome_script"
              required
            />
            <p className="text-xs text-gray-400 mt-1">
              No spaces or special characters. Use underscores.
            </p>
          </div>

          <div>
            <label className="block text-sm font-medium mb-2">
              Description
            </label>
            <input
              type="text"
              value={description}
              onChange={(e) => setDescription(e.target.value)}
              className="w-full bg-gray-700 border border-gray-600 rounded px-3 py-2 focus:outline-none focus:border-blue-500"
              placeholder="What does this script do?"
            />
          </div>

          <div>
            <label className="block text-sm font-medium mb-2">
              Author
            </label>
            <input
              type="text"
              value={author}
              onChange={(e) => setAuthor(e.target.value)}
              className="w-full bg-gray-700 border border-gray-600 rounded px-3 py-2 focus:outline-none focus:border-blue-500"
              placeholder="Your name"
            />
          </div>

          <div>
            <label className="block text-sm font-medium mb-2">
              Python Code *
            </label>
            <textarea
              value={scriptContent}
              onChange={(e) => handleCodeChange(e.target.value)}
              className="w-full bg-gray-900 border border-gray-600 rounded px-3 py-2 font-mono text-sm focus:outline-none focus:border-blue-500"
              rows={15}
              placeholder="# Write your Python code here&#10;import requests&#10;&#10;def main():&#10;    print('Hello, World!')&#10;&#10;if __name__ == '__main__':&#10;    main()"
              required
            />
          </div>

          {detectedDeps.length > 0 && (
            <div className="bg-gray-700 border border-gray-600 rounded p-4">
              <h3 className="font-medium mb-2">🔍 Detected Dependencies</h3>
              <div className="flex flex-wrap gap-2">
                {detectedDeps.map((dep) => (
                  <span
                    key={dep}
                    className="bg-blue-600 text-white px-3 py-1 rounded-full text-sm"
                  >
                    {dep}
                  </span>
                ))}
              </div>
              <p className="text-xs text-gray-400 mt-2">
                These will be added to requirements.txt
              </p>
            </div>
          )}

          <div className="flex gap-3 pt-4">
            <button
              type="submit"
              disabled={isSubmitting}
              className="flex-1 bg-gradient-to-r from-blue-600 to-purple-600 hover:from-blue-700 hover:to-purple-700 disabled:opacity-50 disabled:cursor-not-allowed px-6 py-3 rounded font-medium transition-all"
            >
              {isSubmitting ? (
                <span className="flex items-center justify-center gap-2">
                  <svg className="animate-spin h-5 w-5" viewBox="0 0 24 24">
                    <circle
                      className="opacity-25"
                      cx="12"
                      cy="12"
                      r="10"
                      stroke="currentColor"
                      strokeWidth="4"
                      fill="none"
                    />
                    <path
                      className="opacity-75"
                      fill="currentColor"
                      d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"
                    />
                  </svg>
                  Adding Script...
                </span>
              ) : (
                '✨ Add Script & Push to GitHub'
              )}
            </button>
            <button
              type="button"
              onClick={onClose}
              disabled={isSubmitting}
              className="px-6 py-3 bg-gray-700 hover:bg-gray-600 rounded font-medium transition-colors disabled:opacity-50"
            >
              Cancel
            </button>
          </div>

          <p className="text-xs text-gray-500 text-center">
            Script will be saved locally and automatically pushed to GitHub
          </p>
        </form>
      </motion.div>
    </motion.div>
  );
};
