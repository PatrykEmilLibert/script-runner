import React, { useCallback, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";

interface AdminDropzoneProps {
  onUploaded: () => void;
  scriptsDirOfficial: string;
}

// Simplified uploader: no drag & drop, allows multi-select via dialog or file input
export const AdminDropzone: React.FC<AdminDropzoneProps> = ({ onUploaded, scriptsDirOfficial }) => {
  const [error, setError] = useState("");
  const [isUploading, setIsUploading] = useState(false);
  const [status, setStatus] = useState("Gotowe");

  const logEvent = (msg: string) => {
    console.log("[ADMIN-UPLOAD]", msg);
    setStatus(msg);
  };

  const uploadSingle = async (fileName: string, content: string) => {
    await invoke<string>("add_official_script", {
      fileName,
      fileContent: content,
      description: "Official script",
      author: "admin",
      scriptsDir: scriptsDirOfficial,
    });
  };

  const uploadPath = async (path: string) => {
    await invoke<string>("add_official_script_from_path", {
      path,
      description: "Official script",
      author: "admin",
      scriptsDir: scriptsDirOfficial,
    });
  };

  const handlePaths = useCallback(async (paths: string[] | null) => {
    if (!paths || paths.length === 0) {
      logEvent("Brak plików");
      return;
    }

    const pyPaths = paths.filter((p) => p.toLowerCase().endsWith(".py"));
    if (!pyPaths.length) {
      setError("Tylko pliki .py są akceptowane");
      return;
    }

    setIsUploading(true);
    setError("");
    try {
      for (const path of pyPaths) {
        logEvent(`Wgrywam: ${path}`);
        await uploadPath(path);
      }
      logEvent(`Zakończono wgrywanie (${pyPaths.length})`);
      onUploaded();
    } catch (err) {
      setError(String(err));
      logEvent(`Błąd: ${err}`);
    } finally {
      setIsUploading(false);
    }
  }, [onUploaded, scriptsDirOfficial]);

  const handleFiles = useCallback(async (files: FileList | null) => {
    if (!files || files.length === 0) {
      logEvent("Brak plików");
      return;
    }

    const valid = Array.from(files).filter((f) => f.name.toLowerCase().endsWith(".py"));
    if (!valid.length) {
      setError("Tylko pliki .py są akceptowane");
      return;
    }

    setIsUploading(true);
    setError("");
    try {
      for (const file of valid) {
        const content = await file.text();
        logEvent(`Wgrywam: ${file.name}`);
        await uploadSingle(file.name, content);
      }
      logEvent(`Zakończono wgrywanie (${valid.length})`);
      onUploaded();
    } catch (err) {
      setError(String(err));
      logEvent(`Błąd: ${err}`);
    } finally {
      setIsUploading(false);
    }
  }, [onUploaded, scriptsDirOfficial]);

  const handleSelect = (e: React.ChangeEvent<HTMLInputElement>) => {
    handleFiles(e.target.files);
  };

  const handleDialogSelect = useCallback(async () => {
    try {
      logEvent("Otwieram okno wyboru");
      const result = await open({
        multiple: true,
        filters: [{ name: "Python", extensions: ["py"] }],
      });

      if (!result) {
        logEvent("Anulowano okno dialogowe");
        return;
      }

      if (Array.isArray(result)) {
        await handlePaths(result);
      } else {
        await handlePaths([result]);
      }
    } catch (dialogError) {
      setError(String(dialogError));
      logEvent(`Dialog error: ${dialogError}`);
    }
  }, [handlePaths]);

  return (
    <div className="border-4 border-dashed border-gray-300 dark:border-gray-600 bg-gray-50 dark:bg-gray-800/50 rounded-lg p-8 transition-all duration-200">
      <div className="flex flex-col sm:flex-row items-start sm:items-center justify-between gap-4">
        <div className="flex-1">
          <h3 className="text-lg font-semibold text-gray-800 dark:text-gray-100">📂 Dodaj oficjalne skrypty (.py)</h3>
          <p className="text-sm text-gray-600 dark:text-gray-400">Możesz wskazać wiele plików naraz – trafią do strefy official (read-only)</p>
          <div className="mt-3 p-3 bg-purple-100 dark:bg-purple-900/30 rounded border border-purple-300 dark:border-purple-700">
            <p className="text-purple-800 dark:text-purple-300 text-xs font-mono break-all">Status: {status}</p>
          </div>
          {isUploading && <p className="text-blue-500 text-sm mt-2">⏳ Wgrywam pliki...</p>}
          {error && <p className="text-red-500 dark:text-red-400 text-sm mt-2">❌ {error}</p>}
        </div>
        <div className="flex flex-col gap-2 min-w-[200px]">
          <button
            type="button"
            onClick={handleDialogSelect}
            disabled={isUploading}
            className="px-4 py-2 bg-blue-600 hover:bg-blue-700 rounded text-white cursor-pointer whitespace-nowrap transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
          >
            {isUploading ? "Wgrywam..." : "Wybierz pliki (okno)"}
          </button>
          <label className="px-4 py-2 bg-gray-700 hover:bg-gray-600 rounded text-white cursor-pointer whitespace-nowrap transition-colors disabled:opacity-50 disabled:cursor-not-allowed text-center">
            {isUploading ? "Wgrywam..." : "Wybierz pliki (fallback)"}
            <input type="file" accept=".py" multiple className="hidden" onChange={handleSelect} disabled={isUploading} />
          </label>
        </div>
      </div>
    </div>
  );
};


