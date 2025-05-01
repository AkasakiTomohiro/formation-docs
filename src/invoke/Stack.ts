import { invoke } from '@tauri-apps/api/core';

import type { StackInfo } from '../features/Workspace';
import type { TemplateSummary } from '../features/Workspace/Stack/hooks/UseTemplates';
import type { CommandResult } from '../lib/CommandResult';

export async function loadStacks(): Promise<StackInfo[]> {
  const result = await invoke<CommandResult<StackInfo[]>>(
    'load_stacks_command',
  );
  if (!result.success) {
    throw new Error(result.value);
  }
  return result.value;
}

export async function deleteStack(stackId: string): Promise<void> {
  const result = await invoke<CommandResult<void>>('delete_stack_command', {
    stack_id: stackId,
  });
  if (!result.success) {
    throw new Error(result.value);
  }
  return result.value;
}

export async function importStack(stackFilePath: string): Promise<void> {
  const result = await invoke<CommandResult<void>>('import_stack_command', {
    stack_file_path: stackFilePath,
  });
  if (!result.success) {
    throw new Error(result.value);
  }
  return result.value;
}

export async function loadStack(stackName: string): Promise<StackInfo> {
  const result = await invoke<CommandResult<StackInfo>>('load_stack_command', {
    stack_id: stackName,
  });
  if (!result.success) {
    throw new Error(result.value);
  }
  return result.value;
}

export async function loadTemplateSummary(): Promise<TemplateSummary[]> {
  const result = await invoke<CommandResult<TemplateSummary[]>>(
    'load_template_summary_command',
  );
  if (!result.success) {
    throw new Error(result.value);
  }
  return result.value;
}
