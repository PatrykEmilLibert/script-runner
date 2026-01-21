import { useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { useTranslation } from 'react-i18next';
import { ChevronDown, ChevronUp } from 'lucide-react';

interface Candidate {
  path: string;
  exists: boolean;
  valid: boolean;
}

export default function AdminKeyDiagnostics() {
  const { t } = useTranslation();
  const [expanded, setExpanded] = useState(false);
  const [info, setInfo] = useState<{ candidates: Candidate[] } | null>(null);
  const [loading, setLoading] = useState(false);

  const checkPaths = async () => {
    setLoading(true);
    try {
      const result = await invoke('get_admin_key_info');
      setInfo(result as { candidates: Candidate[] });
      setExpanded(true);
    } catch (e) {
      console.error('Failed to get admin key info:', e);
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="bg-white dark:bg-gray-800 rounded-lg border dark:border-gray-700 p-4 mb-4">
      <button
        onClick={checkPaths}
        disabled={loading}
        className="w-full flex items-center justify-between px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 disabled:bg-gray-400 transition-colors"
      >
        <span>🔐 {t('admin.title')} - Diagnostyka klucza</span>
        {loading ? '⏳' : expanded ? <ChevronUp /> : <ChevronDown />}
      </button>

      {expanded && info && (
        <div className="mt-4 space-y-3">
          {info.candidates.map((candidate, idx) => (
            <div
              key={idx}
              className={`p-3 rounded-lg border-l-4 ${
                candidate.valid
                  ? 'bg-green-50 dark:bg-green-900 border-green-500'
                  : candidate.exists
                  ? 'bg-yellow-50 dark:bg-yellow-900 border-yellow-500'
                  : 'bg-gray-50 dark:bg-gray-700 border-gray-500'
              }`}
            >
              <div className="flex items-start gap-3">
                <span className="text-lg">
                  {candidate.valid ? '✅' : candidate.exists ? '⚠️' : '❌'}
                </span>
                <div className="flex-1">
                  <div className="font-mono text-sm text-gray-800 dark:text-gray-200 break-all">
                    {candidate.path}
                  </div>
                  <div className="text-xs text-gray-600 dark:text-gray-400 mt-1">
                    {candidate.valid
                      ? 'Klucz znaleziony i ważny ✓'
                      : candidate.exists
                      ? 'Plik istnieje, ale nie jest ważny (zły format)'
                      : 'Plik nie istnieje'}
                  </div>
                </div>
              </div>
            </div>
          ))}

          <div className="mt-4 p-3 bg-blue-50 dark:bg-blue-900 rounded-lg text-sm text-gray-700 dark:text-gray-300">
            <strong>Jak to działa?</strong>
            <ol className="mt-2 space-y-1 list-decimal list-inside">
              <li>1. Sprawdza zmienną env: <code className="bg-gray-200 dark:bg-gray-700 px-1 rounded">SR_ADMIN_KEY_PATH</code></li>
              <li>2. Desktop w profilu użytkownika</li>
              <li>3. <code className="bg-gray-200 dark:bg-gray-700 px-1 rounded">USERPROFILE\Desktop</code> (Windows)</li>
              <li>4. <code className="bg-gray-200 dark:bg-gray-700 px-1 rounded">AppData\script-runner</code> (Windows)</li>
              <li>5. <code className="bg-gray-200 dark:bg-gray-700 px-1 rounded">~/Desktop</code> (Linux/macOS)</li>
              <li>6. <code className="bg-gray-200 dark:bg-gray-700 px-1 rounded">/tmp/sr-admin.key</code> (fallback)</li>
            </ol>
          </div>
        </div>
      )}
    </div>
  );
}
