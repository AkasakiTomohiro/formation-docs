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
