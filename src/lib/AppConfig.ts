import { invoke } from '@tauri-apps/api/core';
import { BaseDirectory, writeTextFile } from '@tauri-apps/plugin-fs';
import type { CommandResult } from './CommandResult';

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
  console.log({ result });
  if (result.success) {
    return result.value;
  }
  throw new Error(result.value);
}

/**
 * ワークスペースの設定を保存する
 * @param workspaces
 */
export async function saveAppConfig(workspaces: AppConfig): Promise<void> {
  await writeTextFile(APP_CONFIG_FILE_NAME, JSON.stringify(workspaces), {
    baseDir: BaseDirectory.AppLocalData,
  });
}
