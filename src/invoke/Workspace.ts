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
    'create_workspace',
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
    'load_workspace',
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
  const result =
    await invoke<CommandResult<WorkspaceExpand[]>>('load_workspaces');
  if (!result.success) {
    throw new Error(result.value);
  }
  return result.value;
}

export async function updateWorkspace(
  workspaceId: string,
  workspace: Workspace,
): Promise<void> {
  const result = await invoke<CommandResult<void>>('update_workspace', {
    workspace_id: workspaceId,
    workspace: workspace,
  });
  if (!result.success) {
    throw new Error(result.value);
  }
}
