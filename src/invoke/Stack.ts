import { invoke } from '@tauri-apps/api/core';

import type { Stack } from '../features/Workspace';
import type { CommandResult } from '../lib/CommandResult';

export async function loadStacks(workspaceDirectory: string): Promise<Stack[]> {
  const result = await invoke<CommandResult<Stack[]>>('load_stacks_command', {
    workspace_directory: workspaceDirectory,
  });
  if (!result.success) {
    throw new Error(result.value);
  }
  return result.value;
}

export async function deleteStack(
  workspaceDirectory: string,
  stackName: string,
): Promise<void> {
  const result = await invoke<CommandResult<void>>('delete_stack_command', {
    workspace_directory: workspaceDirectory,
    stack_name: stackName,
  });
  if (!result.success) {
    throw new Error(result.value);
  }
  return result.value;
}

export async function importStack(
  workspaceDirectory: string,
  stackFilePath: string,
): Promise<void> {
  const result = await invoke<CommandResult<void>>('import_stack_command', {
    workspace_directory: workspaceDirectory,
    stack_file_path: stackFilePath,
  });
  if (!result.success) {
    throw new Error(result.value);
  }
  return result.value;
}
