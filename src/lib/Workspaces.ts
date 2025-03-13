import {
  BaseDirectory,
  exists,
  mkdir,
  readTextFile,
  stat,
  writeTextFile,
} from '@tauri-apps/plugin-fs';

export interface Workspace {
  name: string;
  directory: string;
  description: string;
}

export interface Workspaces {
  workspaces: Workspace[];
}

export const WORKSPACE_FILE_NAME = 'workspaces.json';

export async function getWorkspaces(): Promise<Workspaces> {
  const workspaces = await readWorkspaceFiles();
  return workspaces;
}

/**
 * ワークスペースのファイルを読み込む。ない場合は作成する。
 * @param workspaceName
 * @returns
 */
export async function readWorkspaceFiles(): Promise<Workspaces> {
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
  await writeTextFile(WORKSPACE_FILE_NAME, JSON.stringify(result), {
    baseDir: BaseDirectory.AppLocalData,
  });
  return result;
}
