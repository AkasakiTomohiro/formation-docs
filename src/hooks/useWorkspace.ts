import * as path from '@tauri-apps/api/path';
import { useCallback, useEffect, useRef, useState } from 'react';
import { getWorkspaces, saveWorkspaces } from '../lib/Workspaces';

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
   * @returns 追加に成功した場合はtrue
   */
  addWorkspace: (props: AddWorkspace) => Promise<boolean>;

  /**
   * ワークスペースを読み込む
   */
  loadWorkspaces: () => Promise<void>;
};

export function useWorkspace(): UseWorkspaceResult {
  const isFirstRender = useRef(true);
  const [state, setState] = useState<'loading' | 'loaded' | 'error'>('loading');
  const [workspaces, setWorkspaces] = useState<Workspaces>({ workspaces: [] });

  const addWorkspace = useCallback(
    async (props: AddWorkspace): Promise<boolean> => {
      const direname = await path.basename(props.directory);
      const workspace: Workspace = {
        name: direname,
        directory: props.directory,
        description: '',
      };
      if (
        workspaces.workspaces.find((w) => w.directory === workspace.directory)
      ) {
        return false;
      }
      const newWorkspaces: Workspaces = {
        workspaces: [...workspaces.workspaces, workspace],
      };
      setWorkspaces(newWorkspaces);
      await saveWorkspaces(newWorkspaces);
      return true;
    },
    [workspaces],
  );

  const loadWorkspaces = useCallback(async () => {
    setState('loading');
    await new Promise((resolve) => setTimeout(resolve, 300));
    await getWorkspaces()
      .then((workspaces) => {
        setWorkspaces(workspaces);
        setState('loaded');
      })
      .catch((error) => {
        console.error(error);
        setState('error');
      });
  }, []);

  useEffect(() => {
    if (isFirstRender.current) {
      isFirstRender.current = false;
      return;
    }
    loadWorkspaces();
  }, [loadWorkspaces]);
  return { state, workspaces, addWorkspace, loadWorkspaces };
}
