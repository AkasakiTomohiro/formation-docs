import { useCallback, useState } from 'react';
import { deleteWorkspaceFromAppConfig } from '../invoke/AppConfig';
import { createWorkspace, loadWorkspaces } from '../invoke/Workspace';
import type { CreateWorkspaceInfo, WorkspaceInfo } from '../invoke/AppConfig';

export interface Workspace {
  name: string;
  description: string;
}

export type WorkspaceExpand = WorkspaceInfo & Workspace;

export type UseWorkspacesResult = {
  /**
   * ワークスペースの読み込み状態
   */
  state: 'loading' | 'loaded';

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

export function useWorkspaces(): UseWorkspacesResult {
  const [state, setState] = useState<'loading' | 'loaded'>('loading');
  const [workspaces, setWorkspaces] = useState<WorkspaceExpand[]>([]);

  const createWorkspaceWrap = useCallback(
    async (props: CreateWorkspaceInfo): Promise<WorkspaceExpand> => {
      const findWorkspace = workspaces.find((w) => w.directory === props.directory);
      if (findWorkspace) {
        return findWorkspace;
      }
      const newWorkspace = await createWorkspace(props.directory);
      setWorkspaces([...workspaces, newWorkspace]);
      return newWorkspace;
    },
    [workspaces],
  );

  const loadWorkspacesWrap = useCallback(async () => {
    setState('loading');
    await new Promise((resolve) => setTimeout(resolve, 300));
    await loadWorkspaces()
      .then((workspaces) => {
        setWorkspaces(workspaces);
      })
      .finally(() => {
        setState('loaded');
      });
  }, []);

  const deleteWorkspace = useCallback(
    async (selectedItem: WorkspaceInfo) => {
      const newWorkspaces = workspaces.filter((f) => f.id !== selectedItem.id);
      await deleteWorkspaceFromAppConfig(selectedItem.id);
      setWorkspaces(newWorkspaces);
    },
    [workspaces],
  );

  return {
    state,
    workspaces,
    createWorkspace: createWorkspaceWrap,
    loadWorkspaces: loadWorkspacesWrap,
    deleteWorkspace,
  };
}
