import {
  BaseDirectory,
  exists,
  readTextFile,
  writeTextFile,
} from '@tauri-apps/plugin-fs';

export interface Workspace {
  id: string;
  name: string;
  directory: string;
  description: string;
}

export type AddWorkspace = Pick<Workspace, 'directory'>;

export interface Workspaces {
  workspaces: Workspace[];
}

export const WORKSPACE_FILE_NAME = 'workspaces.json';

/**
 * ワークスペースのファイルを読み込む。ない場合は作成する。
 * @param workspaceName
 * @returns
 */
export async function getWorkspaces(): Promise<Workspaces> {
  // ワークスペースファイルの存在確認
  const isWorkspaceFileExists = await exists(WORKSPACE_FILE_NAME, {
    baseDir: BaseDirectory.AppLocalData,
  });
  if (isWorkspaceFileExists) {
    // ワークスペースのファイルを読み込む
    const workspaces = await readTextFile(WORKSPACE_FILE_NAME, {
      baseDir: BaseDirectory.AppLocalData,
    });
    return JSON.parse(workspaces);
  }

  // ワークスペースファイルの新規作成
  const result = { workspaces: [] };
  await saveWorkspaces(result);
  return result;
}

/**
 * ワークスペースの設定を保存する
 * @param workspaces
 */
export async function saveWorkspaces(workspaces: Workspaces): Promise<void> {
  await writeTextFile(WORKSPACE_FILE_NAME, JSON.stringify(workspaces), {
    baseDir: BaseDirectory.AppLocalData,
  });
}
