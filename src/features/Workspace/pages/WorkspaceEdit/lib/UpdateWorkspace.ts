import { invoke } from '@tauri-apps/api/core';

import type { Workspace } from '../../../../../hooks/useWorkspaces';
import type { CommandResult } from '../../../../../lib/CommandResult';

export async function updateWorkspace(workspace: Workspace): Promise<void> {
  const result = await invoke<CommandResult<void>>('update_workspace_command', {
    workspace: workspace,
  });
  if (!result.success) {
    throw new Error(result.value);
  }
}
