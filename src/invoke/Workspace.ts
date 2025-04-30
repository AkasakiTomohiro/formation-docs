import { invoke } from '@tauri-apps/api/core';

import type { CommandResult } from '../lib/CommandResult';
import type { WorkspaceInfo } from './AppConfig';

export interface Workspace {
  name: string;
  description: string;
}
export type WorkspaceExpand = WorkspaceInfo & Workspace;

export const WORKSPACE_FILE_NAME = 'workspace.json';

export async function createWorkspace(
  directory: string,
): Promise<WorkspaceExpand> {
  const result = await invoke<CommandResult<WorkspaceExpand>>(
    'create_workspace_command',
    {
      directory: directory,
    },
  );
  if (!result.success) {
    throw new Error(result.value);
  }
  return result.value;
}

export async function loadWorkspace(
  workspaceId: string,
): Promise<WorkspaceExpand> {
  const result = await invoke<CommandResult<WorkspaceExpand>>(
    'load_workspace_merge_info_command',
    {
      workspace_id: workspaceId,
    },
  );
  if (!result.success) {
    throw new Error(result.value);
  }
  return result.value;
}

export async function loadWorkspaces(): Promise<WorkspaceExpand[]> {
  const result = await invoke<CommandResult<WorkspaceExpand[]>>(
    'load_workspaces_command',
  );
  if (!result.success) {
    throw new Error(result.value);
  }
  return result.value;
}

export async function updateWorkspace(workspace: Workspace): Promise<void> {
  const result = await invoke<CommandResult<void>>('update_workspace_command', {
    workspace: workspace,
  });
  if (!result.success) {
    throw new Error(result.value);
  }
}
