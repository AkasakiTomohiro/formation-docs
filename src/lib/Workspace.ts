import { invoke } from '@tauri-apps/api/core';
import * as path from '@tauri-apps/api/path';

export interface Workspace {
  name: string;
  description: string;
}

export const WORKSPACE_FILE_NAME = 'workspace.json';

export async function createWorkspace(directory: string): Promise<Workspace> {
  const direname = await path.basename(directory);
  const workspace: Workspace = {
    name: direname,
    description: '',
  };
  const result = await invoke<string>('create_workspace', {
    filePath: await path.join(directory, WORKSPACE_FILE_NAME),
    content: JSON.stringify(workspace),
  });
  return JSON.parse(result);
}

export async function loadWorkspace(directory: string): Promise<Workspace> {
  const workspace = await invoke<string>('read_file', {
    filePath: await path.join(directory, WORKSPACE_FILE_NAME),
  });
  return JSON.parse(workspace);
}

export async function saveWorkspace(
  directory: string,
  workspace: Workspace,
): Promise<void> {
  await invoke('save_file', {
    filePath: await path.join(directory, WORKSPACE_FILE_NAME),
    content: JSON.stringify(workspace),
  });
}
