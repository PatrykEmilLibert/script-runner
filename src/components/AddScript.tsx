import React, { useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-dialog';
import { Modal, Button, TextInput, Stack, Group, Alert, Badge, Text, Card } from '@mantine/core';
import { useForm } from '@mantine/form';
import { Upload, FileText } from 'lucide-react';

interface AddScriptProps {
  onScriptAdded: () => void;
  onClose: () => void;
  scriptsDir: string;
}

export const AddScript: React.FC<AddScriptProps> = ({ onScriptAdded, onClose, scriptsDir }) => {
  const [isSubmitting, setIsSubmitting] = useState(false);
  const [selectedFile, setSelectedFile] = useState<{ name: string; path: string } | null>(null);
  const [detectedDeps, setDetectedDeps] = useState<string[]>([]);
  const [isLoadingFile, setIsLoadingFile] = useState(false);

  const form = useForm({
    initialValues: {
      scriptName: '',
      description: '',
      author: '',
    },
    validate: {
      scriptName: (val) => (!val ? 'Script name is required' : null),
    },
  });

  const analyzeCode = (code: string) => {
    const deps = new Set<string>();
    const lines = code.split('\n');
    
    const stdlibModules = new Set([
      'os', 'sys', 're', 'json', 'time', 'datetime', 'random', 'math',
      'collections', 'itertools', 'functools', 'pathlib', 'typing',
      'logging', 'argparse', 'subprocess', 'threading', 'multiprocessing',
      'urllib', 'requests', 'bs4', 'pandas', 'numpy', 'flask', 'django'
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

  const handleSelectFile = async () => {
    try {
      setIsLoadingFile(true);
      const selected = await open({
        filters: [
          {
            name: 'Python',
            extensions: ['py'],
          }
        ],
        multiple: false,
        directory: false,
      });

      if (typeof selected === 'string' && selected) {
        // Get file name from path
        const fileName = selected.split(/[\\/]/).pop() || 'script';
        const scriptNameWithoutExt = fileName.replace('.py', '').replace(/[^a-zA-Z0-9_]/g, '_');
        
        // Read file content via Tauri command
        try {
          const content: string = await invoke('read_file_content', { filePath: selected });
          setSelectedFile({ name: fileName, path: selected });
          form.setFieldValue('scriptName', scriptNameWithoutExt);
          analyzeCode(content);
          // Store content in window for submission
          (window as any).__selectedFileContent = content;
        } catch (readErr) {
          console.error('Failed to read file:', readErr);
          alert('Failed to read file content. Please try again.');
        }
      }
    } catch (err) {
      console.error('Failed to select file:', err);
    } finally {
      setIsLoadingFile(false);
    }
  };

  const handleSubmit = async (values: typeof form.values) => {
    const fileContent = (window as any).__selectedFileContent;
    if (!values.scriptName || !fileContent) {
      return;
    }
    
    setIsSubmitting(true);
    
    try {
      const result = await invoke<string>('add_script', {
        scriptName: values.scriptName.trim(),
        scriptContent: fileContent,
        description: values.description || 'No description provided',
        author: values.author || 'Unknown',
        scriptsDir,
      });
      
      console.log(result);
      (window as any).__selectedFileContent = null;
      onScriptAdded();
      onClose();
    } catch (err) {
      form.setFieldError('scriptName', err as string);
    } finally {
      setIsSubmitting(false);
    }
  };

  return (
    <Modal
      opened={true}
      onClose={onClose}
      title="✨ Add New Script"
      size="md"
      scrollAreaComponent={Stack}
    >
      <form onSubmit={form.onSubmit(handleSubmit)}>
        <Stack gap="md">
          {/* File Selection */}
          {!selectedFile ? (
            <Card 
              p="xl" 
              radius="md" 
              withBorder
              style={{
                border: '2px dashed #FF1493',
                textAlign: 'center',
                cursor: 'pointer',
                transition: 'all 0.2s ease',
              }}
              onClick={handleSelectFile}
              onMouseEnter={(e) => {
                (e.currentTarget as HTMLElement).style.background = 'rgba(255, 20, 147, 0.05)';
              }}
              onMouseLeave={(e) => {
                (e.currentTarget as HTMLElement).style.background = 'transparent';
              }}
            >
              <Stack gap="sm" align="center">
                <Upload size={32} color="#FF1493" />
                <div>
                  <Text fw={500}>{isLoadingFile ? 'Loading...' : 'Click to select Python file'}</Text>
                  <Text size="sm" c="dimmed">.py files only</Text>
                </div>
              </Stack>
            </Card>
          ) : (
            <Card p="md" radius="md" withBorder className="glass-pink">
              <Group justify="space-between" mb="sm">
                <Group gap="xs">
                  <FileText size={20} color="#FF1493" />
                  <div>
                    <Text size="sm" fw={500}>{selectedFile.name}</Text>
                  </div>
                </Group>
                <Button
                  size="xs"
                  variant="light"
                  color="pink"
                  onClick={handleSelectFile}
                  loading={isLoadingFile}
                >
                  Change
                </Button>
              </Group>
            </Card>
          )}

          <TextInput
            label="Script Name *"
            placeholder="my_awesome_script"
            description="Auto-filled from filename. Adjust if needed."
            {...form.getInputProps('scriptName')}
            required
            disabled={!selectedFile}
          />

          <TextInput
            label="Description"
            placeholder="What does this script do?"
            {...form.getInputProps('description')}
            disabled={!selectedFile}
          />

          <TextInput
            label="Author"
            placeholder="Your name"
            {...form.getInputProps('author')}
            disabled={!selectedFile}
          />

          {detectedDeps.length > 0 && (
            <Stack gap="sm">
              <Text fw={500} size="sm">🔍 Detected Dependencies</Text>
              <Group gap="xs" wrap="wrap">
                {detectedDeps.map((dep) => (
                  <Badge key={dep} color="pink" variant="light">
                    {dep}
                  </Badge>
                ))}
              </Group>
              <Text size="xs" c="dimmed">
                These will be added to requirements.txt
              </Text>
            </Stack>
          )}

          <Group justify="flex-end" mt="md">
            <Button variant="default" onClick={onClose} disabled={isSubmitting}>
              Cancel
            </Button>
            <Button
              type="submit"
              loading={isSubmitting}
              disabled={!selectedFile}
              variant="gradient"
              gradient={{ from: "pink", to: "grape" }}
            >
              ✨ Add Script & Push
            </Button>
          </Group>

          <Text size="xs" c="dimmed" ta="center">
            Script will be saved locally and pushed to GitHub
          </Text>
        </Stack>
      </form>
    </Modal>
  );
};
