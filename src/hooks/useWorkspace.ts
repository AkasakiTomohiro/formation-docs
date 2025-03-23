import * as path from '@tauri-apps/api/path';
import * as logger from '@tauri-apps/plugin-log';
import { useCallback, useState } from 'react';
import { getWorkspaces, saveWorkspaces } from '../lib/Workspaces';

import { v4 as uuidv4 } from 'uuid';
import type { AddWorkspace, Workspace, Workspaces } from '../lib/Workspaces';

export type UseWorkspaceResult = {
  /**
   * ワークスペースの読み込み状態
   */
  state: 'initial' | 'loading' | 'loaded' | 'error';

  /**
   * ワークスペース一覧
   */
  workspaces: Workspaces;

  /**
   * ワークスペースを追加する
   */
  addWorkspace: (props: AddWorkspace) => Promise<Workspace>;

  /**
   * ワークスペースを読み込む
   */
  loadWorkspaces: () => Promise<void>;

  /**
   * ワークスペースを削除する
   */
  deleteWorkspace: (selectedWorkspace: Workspace) => Promise<void>;
};

export function useWorkspace(): UseWorkspaceResult {
  const [state, setState] = useState<'loading' | 'loaded'>('loading');
  const [workspaces, setWorkspaces] = useState<Workspaces>({ workspaces: [] });

  const addWorkspace = useCallback(
    async (props: AddWorkspace): Promise<Workspace> => {
      const direname = await path.basename(props.directory);
      const workspace: Workspace = {
        id: uuidv4(),
        name: direname,
        directory: props.directory,
        description: '',
      };
      const findWorkspace = workspaces.workspaces.find(
        (w) => w.directory === workspace.directory,
      );
      if (findWorkspace) {
        return findWorkspace;
      }
      const newWorkspaces: Workspaces = {
        workspaces: [...workspaces.workspaces, workspace],
      };
      setWorkspaces(newWorkspaces);
      await saveWorkspaces(newWorkspaces);
      return workspace;
    },
    [workspaces],
  );

  const loadWorkspaces = useCallback(async () => {
    setState('loading');
    await new Promise((resolve) => setTimeout(resolve, 300));
    await getWorkspaces()
      .then((workspaces) => {
        setWorkspaces(workspaces);
      })
      .catch((error) => {
        logger.error(error);
        throw error;
      })
      .finally(() => {
        setState('loaded');
      });
  }, []);

  const deleteWorkspace = useCallback(
    async (selectedItem: Workspace) => {
      const newWorkspaces = {
        workspaces: workspaces.workspaces.filter(
          (f) => f.id !== selectedItem.id,
        ),
      };
      setWorkspaces(newWorkspaces);
      await saveWorkspaces(newWorkspaces);
    },
    [workspaces],
  );

  return { state, workspaces, addWorkspace, loadWorkspaces, deleteWorkspace };
}
