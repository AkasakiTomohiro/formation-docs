import {
  BaseDirectory,
  exists,
  readTextFile,
  writeTextFile,
} from '@tauri-apps/plugin-fs';

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
  // ワークスペースファイルの存在確認
  const isWorkspaceFileExists = await exists(APP_CONFIG_FILE_NAME, {
    baseDir: BaseDirectory.AppLocalData,
  });
  if (isWorkspaceFileExists) {
    // ワークスペースのファイルを読み込む
    const workspaces = await readTextFile(APP_CONFIG_FILE_NAME, {
      baseDir: BaseDirectory.AppLocalData,
    });
    return JSON.parse(workspaces);
  }

  // ワークスペースファイルの新規作成
  const result = { workspaces: [] };
  await saveAppConfig(result);
  return result;
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
