import { Search as SearchIcon, X } from 'lucide-react';
import { useState } from 'react';
import { useTranslation } from 'react-i18next';

interface SearchBoxProps {
  onSearch: (query: string) => void;
}

export default function SearchBox({ onSearch }: SearchBoxProps) {
  const [query, setQuery] = useState('');
  const { t } = useTranslation();

  const handleChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const q = e.target.value;
    setQuery(q);
    onSearch(q);
  };

  const clear = () => {
    setQuery('');
    onSearch('');
  };

  return (
    <div className="relative mb-4">
      <div className="flex items-center bg-gray-100 dark:bg-gray-700 rounded-lg px-3 py-2">
        <SearchIcon size={18} className="text-gray-500 dark:text-gray-400 mr-2" />
        <input
          type="text"
          value={query}
          onChange={handleChange}
          placeholder={t('history.filterBy')}
          className="flex-1 bg-transparent outline-none text-gray-800 dark:text-gray-200 placeholder-gray-500 dark:placeholder-gray-400"
        />
        {query && (
          <button
            onClick={clear}
            className="p-1 hover:bg-gray-200 dark:hover:bg-gray-600 rounded"
            title={t('buttons.clear')}
          >
            <X size={16} className="text-gray-500 dark:text-gray-400" />
          </button>
        )}
      </div>
    </div>
  );
}
