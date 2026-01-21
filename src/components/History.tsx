import { useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { useTranslation } from 'react-i18next';
import SearchBox from './SearchBox';
import { Download, ChevronDown, ChevronUp } from 'lucide-react';

interface RunRecord {
  id: string;
  script_name: string;
  start_time: string;
  end_time: string;
  duration_ms: number;
  status: string;
  output: string;
  error?: string;
}

export default function History() {
  const { t } = useTranslation();
  const [records, setRecords] = useState<RunRecord[]>([]);
  const [filtered, setFiltered] = useState<RunRecord[]>([]);
  const [expanded, setExpanded] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    loadHistory();
    const interval = setInterval(loadHistory, 5000);
    return () => clearInterval(interval);
  }, []);

  const loadHistory = async () => {
    try {
      const result = await invoke('get_run_history', { limit: 100 });
      setRecords(result as RunRecord[]);
      setFiltered(result as RunRecord[]);
    } catch (e) {
      console.error('Failed to load history:', e);
    } finally {
      setLoading(false);
    }
  };

  const handleSearch = (query: string) => {
    if (!query) {
      setFiltered(records);
    } else {
      setFiltered(records.filter(r =>
        r.script_name.toLowerCase().includes(query.toLowerCase())
      ));
    }
  };

  const exportCsv = async () => {
    try {
      const csv = await invoke('export_history_as_csv', { limit: 100 });
      const blob = new Blob([csv as string], { type: 'text/csv;charset=utf-8;' });
      const link = document.createElement('a');
      const url = URL.createObjectURL(blob);
      link.setAttribute('href', url);
      link.setAttribute('download', `historia_${new Date().toISOString().slice(0, 10)}.csv`);
      link.click();
    } catch (e) {
      console.error('Failed to export CSV:', e);
    }
  };

  const formatDuration = (ms: number) => {
    if (ms < 1000) return `${ms}ms`;
    return `${(ms / 1000).toFixed(1)}s`;
  };

  const formatTime = (iso: string) => {
    try {
      const date = new Date(iso);
      return date.toLocaleString('pl-PL');
    } catch {
      return iso;
    }
  };

  if (loading) {
    return <div className="p-4 text-center text-gray-600 dark:text-gray-400">{t('app.loading')}</div>;
  }

  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between mb-4">
        <h2 className="text-2xl font-bold text-gray-800 dark:text-gray-100">{t('history.title')}</h2>
        <button
          onClick={exportCsv}
          disabled={filtered.length === 0}
          className="flex items-center gap-2 px-3 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 disabled:bg-gray-400"
        >
          <Download size={16} />
          {t('history.csv')}
        </button>
      </div>

      <SearchBox onSearch={handleSearch} />

      {filtered.length === 0 ? (
        <div className="p-8 text-center text-gray-500 dark:text-gray-400">
          {records.length === 0 ? t('history.noHistory') : t('history.noHistory')}
        </div>
      ) : (
        <div className="space-y-2">
          {filtered.map(record => (
            <div
              key={record.id}
              className="bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 overflow-hidden"
            >
              <button
                onClick={() => setExpanded(expanded === record.id ? null : record.id)}
                className="w-full p-4 text-left hover:bg-gray-50 dark:hover:bg-gray-700 transition-colors flex items-center justify-between"
              >
                <div className="flex-1">
                  <div className="flex items-center gap-4">
                    <span className="font-medium text-gray-800 dark:text-gray-200 min-w-40">
                      {record.script_name}
                    </span>
                    <span
                      className={`px-2 py-1 rounded text-sm font-semibold ${
                        record.status === 'success'
                          ? 'bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-200'
                          : 'bg-red-100 text-red-800 dark:bg-red-900 dark:text-red-200'
                      }`}
                    >
                      {record.status === 'success' ? t('history.success') : t('history.failed')}
                    </span>
                    <span className="text-sm text-gray-600 dark:text-gray-400 min-w-40">
                      {formatTime(record.start_time)}
                    </span>
                    <span className="text-sm text-gray-500 dark:text-gray-400">
                      {formatDuration(record.duration_ms)}
                    </span>
                  </div>
                </div>
                {expanded === record.id ? <ChevronUp size={20} /> : <ChevronDown size={20} />}
              </button>

              {expanded === record.id && (
                <div className="px-4 pb-4 border-t border-gray-200 dark:border-gray-700 bg-gray-50 dark:bg-gray-900">
                  {record.output && (
                    <div className="mb-4">
                      <h4 className="font-semibold text-gray-700 dark:text-gray-300 mb-2">
                        {t('dashboard.output')}
                      </h4>
                      <pre className="bg-gray-800 text-gray-100 p-3 rounded text-sm overflow-x-auto max-h-60 overflow-y-auto">
                        {record.output}
                      </pre>
                    </div>
                  )}
                  {record.error && (
                    <div className="mb-4">
                      <h4 className="font-semibold text-red-700 dark:text-red-300 mb-2">
                        {t('app.error')}
                      </h4>
                      <pre className="bg-red-900 text-red-100 p-3 rounded text-sm overflow-x-auto max-h-40 overflow-y-auto">
                        {record.error}
                      </pre>
                    </div>
                  )}
                </div>
              )}
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
