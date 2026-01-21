import { useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { useTranslation } from 'react-i18next';
import { Key, Copy, Check } from 'lucide-react';

interface GenerateAdminKeyProps {
  onGenerated: () => void;
}

export default function GenerateAdminKey({ onGenerated }: GenerateAdminKeyProps) {
  const { t } = useTranslation();
  const [loading, setLoading] = useState(false);
  const [success, setSuccess] = useState(false);
  const [keyPath, setKeyPath] = useState<string | null>(null);
  const [copied, setCopied] = useState(false);

  const handleGenerate = async () => {
    setLoading(true);
    try {
      const path: string = await invoke('generate_admin_key');
      setKeyPath(path);
      setSuccess(true);
      setTimeout(() => onGenerated(), 1500);
    } catch (e) {
      console.error('Failed to generate key:', e);
      alert(`Błąd: ${e}`);
    } finally {
      setLoading(false);
    }
  };

  const copyToClipboard = () => {
    if (keyPath) {
      navigator.clipboard.writeText(keyPath);
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    }
  };

  return (
    <div className="w-full max-w-2xl mx-auto p-8">
      <div className="bg-white dark:bg-gray-800 rounded-lg border-2 border-blue-600 dark:border-blue-500 p-8">
        <div className="flex items-center justify-center mb-6">
          <Key size={48} className="text-blue-600 dark:text-blue-400" />
        </div>

        <h1 className="text-3xl font-bold text-center text-gray-800 dark:text-gray-100 mb-4">
          🔐 Wygeneruj klucz administratora
        </h1>

        <p className="text-center text-gray-600 dark:text-gray-400 mb-8">
          Klucz administratora pozwala na zarządzanie skryptami oficjalnymi i ustawieniami systemu.
        </p>

        {success && keyPath ? (
          <div className="space-y-4">
            <div className="bg-green-100 dark:bg-green-900 border border-green-500 rounded-lg p-4">
              <p className="text-green-800 dark:text-green-200 font-semibold mb-3">
                ✅ Klucz został wygenerowany!
              </p>
              <p className="text-sm text-green-700 dark:text-green-300 mb-3">
                Klucz znajduje się w:
              </p>
              <div className="bg-green-50 dark:bg-green-950 p-3 rounded border border-green-300 dark:border-green-700 font-mono text-sm break-all mb-3">
                {keyPath}
              </div>
              <button
                onClick={copyToClipboard}
                className="flex items-center gap-2 w-full px-4 py-2 bg-green-600 hover:bg-green-700 text-white rounded-lg transition-colors justify-center"
              >
                {copied ? <Check size={18} /> : <Copy size={18} />}
                {copied ? 'Skopiowano!' : 'Kopiuj ścieżkę'}
              </button>
            </div>

            <div className="bg-blue-50 dark:bg-blue-900 border border-blue-300 dark:border-blue-700 rounded-lg p-4 text-sm text-blue-800 dark:text-blue-200">
              <strong>Instrukcja:</strong>
              <ol className="mt-2 space-y-2 list-decimal list-inside">
                <li>Klucz jest zapisany w pliku JSON na Twoim pulpicie</li>
                <li>Trzymaj go w bezpiecznym miejscu</li>
                <li>Możesz skopiować go na inne komputery (Ubuntu, macOS)</li>
                <li>Umieść go w folderze <code className="bg-gray-200 dark:bg-gray-700 px-1 rounded">~/Desktop</code>
                </li>
              </ol>
            </div>

            <p className="text-center text-gray-500 dark:text-gray-400 text-sm">
              Aplikacja zaraz się przeładuje...
            </p>
          </div>
        ) : (
          <div className="space-y-4">
            <div className="bg-yellow-50 dark:bg-yellow-900 border border-yellow-300 dark:border-yellow-700 rounded-lg p-4">
              <p className="text-yellow-800 dark:text-yellow-200 text-sm">
                ⚠️ Nie znaleziono klucza administratora. Wygeneruj nowy, aby uzyskać dostęp do funkcji administratora.
              </p>
            </div>

            <button
              onClick={handleGenerate}
              disabled={loading}
              className="w-full px-6 py-3 bg-blue-600 hover:bg-blue-700 disabled:bg-gray-400 text-white font-bold rounded-lg transition-colors text-lg"
            >
              {loading ? '⏳ Generowanie...' : '🔑 Wygeneruj klucz administratora'}
            </button>

            <p className="text-center text-sm text-gray-500 dark:text-gray-400">
              Klucz będzie zapisany na Twoim pulpicie (Desktop)
            </p>
          </div>
        )}
      </div>
    </div>
  );
}
