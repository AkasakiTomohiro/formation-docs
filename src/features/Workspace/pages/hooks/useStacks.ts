import { useCallback, useState } from 'react';
import { useOutletContext } from 'react-router';

import { deleteStack, importStack, loadStacks } from '../../../../invoke/Stack';

import type { WorkspaceLayoutLoaderData } from '../../Loader';
import type { StackInfo } from '../WorkspaceHome';

export type UseStacksResult = {
  /**
   * スタックの読み込み状態
   */
  state: 'loading' | 'loaded';

  /**
   * スタック一覧
   */
  stacks: StackInfo[];

  /**
   * スタックをインポートする
   */
  importStack: (stackFilePath: string) => Promise<void>;

  /**
   * スタックを読み込む
   */
  loadStacks: () => Promise<void>;

  /**
   * スタックを削除する
   */
  deleteStack: (selectedStack: StackInfo) => Promise<void>;
};

export function useStacks(): UseStacksResult {
  const [state, setState] = useState<'loading' | 'loaded'>('loading');
  const [stacks, setStacks] = useState<StackInfo[]>([]);
  const workspace = useOutletContext<WorkspaceLayoutLoaderData>();

  const importStackWrap = useCallback(
    async (stackFilePath: string) => {
      await importStack(workspace.directory, stackFilePath);
    },
    [workspace.directory],
  );

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
    async (selectedStack: StackInfo) => {
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
    importStack: importStackWrap,
    loadStacks: loadStacksWrap,
    deleteStack: deleteStackWrap,
  };
}
