import { useCallback, useState } from 'react';
import { useOutletContext } from 'react-router';

import { deleteStack, loadStacks } from '../../../../invoke/Stack';

import type { Stack } from '../WorkspaceHome';
import type { WorkspaceLayoutLoaderData } from '../../Loader';

export type UseStacksResult = {
  /**
   * スタックの読み込み状態
   */
  state: 'loading' | 'loaded';

  /**
   * スタック一覧
   */
  stacks: Stack[];

  /**
   * スタックをインポートする
   */
  importStack: () => Promise<void>;

  /**
   * スタックを読み込む
   */
  loadStacks: () => Promise<void>;

  /**
   * スタックを削除する
   */
  deleteStack: (selectedStack: Stack) => Promise<void>;
};

export function useStacks(): UseStacksResult {
  const [state, setState] = useState<'loading' | 'loaded'>('loading');
  const [stacks, setStacks] = useState<Stack[]>([]);
  const workspace = useOutletContext<WorkspaceLayoutLoaderData>();

  const importStack = useCallback(async () => {}, []);

  const loadStacksWrap = useCallback(async () => {
    setState('loading');
    await new Promise((resolve) => setTimeout(resolve, 300));
    await loadStacks(workspace.directory)
      .then((stacks) => {
        setStacks(stacks);
      })
      .finally(() => {
        setState('loaded');
      });
  }, [workspace]);

  const deleteStackWrap = useCallback(
    async (selectedStack: Stack) => {
      const newStacks = stacks.filter(
        (stack) => stack.name !== selectedStack.name,
      );
      await deleteStack(workspace.directory, selectedStack.name);
      setStacks(newStacks);
    },
    [workspace, stacks],
  );

  return {
    state,
    stacks,
    importStack,
    loadStacks: loadStacksWrap,
    deleteStack: deleteStackWrap,
  };
}
