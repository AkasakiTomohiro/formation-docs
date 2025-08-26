import { invoke } from '@tauri-apps/api/core';

import type { CommandResult } from '../../../lib/CommandResult';

export async function openWorkspace(workspaceId: string): Promise<boolean> {
  const result = await invoke<CommandResult<boolean>>('open_workspace_command', {
    id: workspaceId,
  });
  if (!result.success) {
    throw new Error(result.value);
  }
  return result.value;
}
