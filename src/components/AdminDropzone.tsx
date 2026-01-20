import React, { useCallback, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { motion } from "framer-motion";

interface AdminDropzoneProps {
  onUploaded: () => void;
  scriptsDirOfficial: string;
}

export const AdminDropzone: React.FC<AdminDropzoneProps> = ({ onUploaded, scriptsDirOfficial }) => {
  const [isDragging, setIsDragging] = useState(false);
  const [error, setError] = useState("");
  const [isUploading, setIsUploading] = useState(false);

  const handleFiles = useCallback(async (files: FileList | null) => {
    if (!files || files.length === 0) return;

    const file = files[0];
    if (!file.name.endsWith(".py")) {
      setError("Tylko pliki .py są akceptowane");
      return;
    }

    setError("");
    setIsUploading(true);
    try {
      const content = await file.text();
      await invoke<string>("add_official_script", {
        fileName: file.name,
        fileContent: content,
        description: "Official script",
        author: "admin",
        scriptsDir: scriptsDirOfficial,
      });
      onUploaded();
    } catch (err) {
      setError(String(err));
    } finally {
      setIsUploading(false);
    }
  }, [onUploaded, scriptsDirOfficial]);

  const onDrop = (e: React.DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
    setIsDragging(false);
    handleFiles(e.dataTransfer.files);
  };

  const onDragOver = (e: React.DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
    setIsDragging(true);
  };

  const onDragLeave = (e: React.DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
    setIsDragging(false);
  };

  const handleSelect = (e: React.ChangeEvent<HTMLInputElement>) => {
    handleFiles(e.target.files);
  };

  return (
    <motion.div
      className={`border-2 border-dashed rounded-lg p-6 bg-gray-800 ${isDragging ? "border-blue-400 bg-gray-700" : "border-gray-600"}`}
      onDrop={onDrop}
      onDragOver={onDragOver}
      onDragLeave={onDragLeave}
      initial={{ opacity: 0 }}
      animate={{ opacity: 1 }}
    >
      <div className="flex items-center justify-between gap-4">
        <div>
          <h3 className="text-lg font-semibold">📂 Drag & Drop official .py</h3>
          <p className="text-sm text-gray-400">Plik trafi do strefy official (read-only dla użytkowników)</p>
          {error && <p className="text-red-400 text-sm mt-2">{error}</p>}
        </div>
        <label className="px-4 py-2 bg-blue-600 hover:bg-blue-700 rounded text-white cursor-pointer">
          {isUploading ? "Wgrywam..." : "Wybierz plik"}
          <input type="file" accept=".py" className="hidden" onChange={handleSelect} disabled={isUploading} />
        </label>
      </div>
    </motion.div>
  );
};
