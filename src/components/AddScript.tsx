import React, { useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { Modal, Button, TextInput, Textarea, Stack, Group, Alert, Badge, Text } from '@mantine/core';
import { useForm } from '@mantine/form';

interface AddScriptProps {
  onScriptAdded: () => void;
  onClose: () => void;
  scriptsDir: string;
}

export const AddScript: React.FC<AddScriptProps> = ({ onScriptAdded, onClose, scriptsDir }) => {
  const [isSubmitting, setIsSubmitting] = useState(false);
  const [detectedDeps, setDetectedDeps] = useState<string[]>([]);

  const form = useForm({
    initialValues: {
      scriptName: '',
      description: '',
      author: '',
      scriptContent: '',
    },
    validate: {
      scriptName: (val) => (!val ? 'Script name is required' : null),
      scriptContent: (val) => (!val ? 'Script content is required' : null),
    },
  });

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
    form.setFieldValue('scriptContent', code);
    analyzeCode(code);
  };

  const handleSubmit = async (values: typeof form.values) => {
    if (!values.scriptName || !values.scriptContent) {
      return;
    }
    
    setIsSubmitting(true);
    
    try {
      const result = await invoke<string>('add_script', {
        scriptName: values.scriptName.trim(),
        scriptContent: values.scriptContent,
        description: values.description || 'No description provided',
        author: values.author || 'Unknown',
        scriptsDir,
      });
      
      console.log(result);
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
      size="lg"
      scrollAreaComponent={Stack}
    >
      <form onSubmit={form.onSubmit(handleSubmit)}>
        <Stack gap="md">
          {form.errors.scriptName && (
            <Alert color="red" title="Error">
              {form.errors.scriptName}
            </Alert>
          )}

          <TextInput
            label="Script Name *"
            placeholder="my_awesome_script"
            description="No spaces or special characters. Use underscores."
            {...form.getInputProps('scriptName')}
            required
          />

          <TextInput
            label="Description"
            placeholder="What does this script do?"
            {...form.getInputProps('description')}
          />

          <TextInput
            label="Author"
            placeholder="Your name"
            {...form.getInputProps('author')}
          />

          <Textarea
            label="Python Code *"
            placeholder={`# Write your Python code here
import requests

def main():
    print('Hello, World!')

if __name__ == '__main__':
    main()`}
            minRows={15}
            {...form.getInputProps('scriptContent')}
            onChange={(e) => handleCodeChange(e.target.value)}
            required
          />

          {detectedDeps.length > 0 && (
            <Stack gap="sm">
              <Text fw={500}>🔍 Detected Dependencies</Text>
              <Group>
                {detectedDeps.map((dep) => (
                  <Badge key={dep} color="blue">
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
              color="blue"
            >
              ✨ Add Script & Push to GitHub
            </Button>
          </Group>

          <Text size="xs" c="dimmed" ta="center">
            Script will be saved locally and automatically pushed to GitHub
          </Text>
        </Stack>
      </form>
    </Modal>
  );
};
