import { useCallback, useState } from 'react';
import { loadAppConfig, saveAppConfig } from '../lib/AppConfig';

import { v4 as uuidv4 } from 'uuid';
import type { CreateWorkspaceInfo, WorkspaceInfo } from '../lib/AppConfig';
import { createWorkspace, loadWorkspace } from '../lib/Workspace';
import type { Workspace } from '../lib/Workspace';

export type UseWorkspaceResult = {
  /**
   * ワークスペースの読み込み状態
   */
  state: 'initial' | 'loading' | 'loaded' | 'error';

  /**
   * ワークスペース一覧
   */
  workspaces: WorkspaceExpand[];

  /**
   * ワークスペースを追加する
   */
  createWorkspace: (props: CreateWorkspaceInfo) => Promise<WorkspaceExpand>;

  /**
   * ワークスペースを読み込む
   */
  loadWorkspaces: () => Promise<void>;

  /**
   * ワークスペースを削除する
   */
  deleteWorkspace: (selectedWorkspace: WorkspaceExpand) => Promise<void>;
};

export type WorkspaceExpand = WorkspaceInfo & Workspace;

export function useWorkspaces(): UseWorkspaceResult {
  const [state, setState] = useState<'loading' | 'loaded'>('loading');
  const [workspaces, setWorkspaces] = useState<WorkspaceExpand[]>([]);

  const createWorkspaceWrap = useCallback(
    async (props: CreateWorkspaceInfo): Promise<WorkspaceExpand> => {
      const findWorkspace = workspaces.find(
        (w) => w.directory === props.directory,
      );
      if (findWorkspace) {
        return findWorkspace;
      }
      const newWorkspace = await createWorkspace(props.directory);
      const workspaceInfo: WorkspaceExpand = {
        ...newWorkspace,
        id: uuidv4(),
        directory: props.directory,
      };
      const newWorkspaces = [...workspaces, workspaceInfo];
      setWorkspaces(newWorkspaces);
      await saveAppConfig({
        workspaces: newWorkspaces.map((m) => ({
          id: m.id,
          directory: m.directory,
        })),
      });
      return workspaceInfo;
    },
    [workspaces],
  );

  const loadWorkspaces = useCallback(async () => {
    setState('loading');
    await new Promise((resolve) => setTimeout(resolve, 300));
    await loadAppConfig()
      .then(async (appConfig) => {
        const mergedWorkspaces: WorkspaceInfo[] = [];
        for (const workspace of appConfig.workspaces) {
          const data = await loadWorkspace(workspace.directory);
          mergedWorkspaces.push({
            ...workspace,
            ...data,
          });
        }
        setWorkspaces(workspaces);
      })
      .catch((error) => {
        console.log(error);
        throw error;
      })
      .finally(() => {
        setState('loaded');
      });
  }, [workspaces]);

  const deleteWorkspace = useCallback(
    async (selectedItem: WorkspaceInfo) => {
      const newWorkspaces = workspaces.filter((f) => f.id !== selectedItem.id);
      setWorkspaces(newWorkspaces);
      await saveAppConfig({
        workspaces: newWorkspaces.map((m) => ({
          id: m.id,
          directory: m.directory,
        })),
      });
    },
    [workspaces],
  );

  return {
    state,
    workspaces,
    createWorkspace: createWorkspaceWrap,
    loadWorkspaces,
    deleteWorkspace,
  };
}
