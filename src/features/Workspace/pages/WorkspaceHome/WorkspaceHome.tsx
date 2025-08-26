import { useCallback, useEffect, useState } from 'react';
import { useNavigate, useRouteLoaderData } from 'react-router';

import { useCollection } from '@cloudscape-design/collection-hooks';
import { open } from '@tauri-apps/plugin-dialog';

import { useFlashbarContext } from '../../../../contexts/FlashbarContext';
import { useWorkspaceResourceContext } from '../../contexts';
import { WorkspaceHomePresentation } from './WorkspaceHome.presentation';
import { deleteStack } from './lib/DeleteStack';
import { importStack } from './lib/ImportStack';
import { loadStacks } from './lib/LoadStacks';

import type { WorkspaceLayoutLoaderData } from '../../Loader';
import type { StackInfo } from '../../lib';

export const WorkspaceHome = (): JSX.Element => {
  const navigate = useNavigate();
  const workspace = useRouteLoaderData('workspace') as WorkspaceLayoutLoaderData;
  const { loadSideMenu } = useWorkspaceResourceContext();
  const { addFlashbarItem } = useFlashbarContext();
  const [selectedItems, setSelectedItems] = useState<StackInfo[]>([]);
  const [state, setState] = useState<'loading' | 'loaded'>('loading');
  const [stacks, setStacks] = useState<StackInfo[]>([]);
  const tableCollection = useCollection<StackInfo>(stacks, {
    pagination: { pageSize: 10 },
  });

  const loadStacksWrap = useCallback(async () => {
    setState('loading');
    await new Promise((resolve) => setTimeout(resolve, 300));
    await loadStacks()
      .then((stacks) => {
        setStacks(stacks);
      })
      .finally(() => {
        setState('loaded');
      });
  }, []);

  const importStackWrap = useCallback(async () => {
    const selectedFile = await open({
      multiple: false,
      directory: false,
      defaultPath: workspace.directory,
      filters: [
        {
          name: 'Template files',
          extensions: ['json', 'yaml', 'yml'],
        },
      ],
    });
    if (selectedFile !== null) {
      importStack(selectedFile)
        .then(async () => {
          await loadStacksWrap();
          await loadSideMenu();
        })
        .catch((error) => {
          addFlashbarItem({
            type: 'error',
            header: '新規スタックのインポートに失敗しました',
            content: typeof error === 'string' ? error : undefined,
          });
        });
    }
  }, [workspace, loadSideMenu, loadStacksWrap, addFlashbarItem]);

  // biome-ignore lint/correctness/useExhaustiveDependencies: <explanation>
  useEffect(() => {
    loadStacksWrap().catch((error) => {
      addFlashbarItem({
        type: 'error',
        header: 'スタックの読み込みに失敗しました',
        content: typeof error === 'string' ? error : undefined,
      });
    });
  }, []);

  const onClickStackName = (item: StackInfo) => () => {
    navigate(`/workspaces/${workspace.id}/resources`, {
      state: {
        selectedStackId: item.id,
        selectedStackName: item.name,
      },
    });
  };

  const onClickDeleteStack = async () => {
    await deleteStack(selectedItems[0].id);
    setStacks((oldStacks) => oldStacks.filter((stack) => stack.id !== selectedItems[0].id));
    setSelectedItems([]);

    await loadSideMenu();
  };

  return (
    <WorkspaceHomePresentation
      selectedItems={selectedItems}
      isLoading={state === 'loading'}
      onSelectionChange={({ detail }) => setSelectedItems(detail.selectedItems)}
      onClickReloadStack={loadStacksWrap}
      tableCollection={tableCollection}
      onClickStackName={onClickStackName}
      onClickImportStack={importStackWrap}
      onClickDeleteStack={onClickDeleteStack}
      onClickWorkspaceEdit={() => navigate(`/workspaces/${workspace.id}/edit`)}
    />
  );
};
