import { invoke } from '@tauri-apps/api/core';
import type { CommandResult } from '../lib/CommandResult';

export type CreateWorkspaceInfo = Pick<WorkspaceInfo, 'directory'>;

export type WorkspaceInfo = {
  id: string;
  directory: string;
};

export type AppConfig = {
  workspaces: WorkspaceInfo[];
};

export interface WorkspacesFile {
  workspaces: Pick<WorkspaceInfo, 'id' | 'directory'>[];
}

export const APP_CONFIG_FILE_NAME = 'app_config.json';

/**
 * ワークスペースのファイルを読み込む。ない場合は作成する。
 * @param workspaceName
 * @returns
 */
export async function loadAppConfig(): Promise<AppConfig> {
  const result = await invoke<CommandResult<AppConfig>>('read_app_config');
  if (result.success) {
    return result.value;
  }
  throw new Error(result.value);
}

/**
 * AppConfigからワークスペースを削除する
 */
export async function deleteWorkspaceFromAppConfig(
  workspaceId: string,
): Promise<void> {
  const result = await invoke<CommandResult<[]>>(
    'delete_workspace_from_app_config',
    {
      workspace_id: workspaceId,
    },
  );
  if (!result.success) {
    throw new Error(result.value);
  }
}
