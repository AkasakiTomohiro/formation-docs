import { useCallback, useEffect, useState } from 'react';
import { useNavigate, useOutletContext } from 'react-router';
import { v4 as uuidV4 } from 'uuid';

import { useCollection } from '@cloudscape-design/collection-hooks';
import { open } from '@tauri-apps/plugin-dialog';

import { useWorkspaceResourceContext } from '../../contexts';
import { WorkspaceHomePresentation } from './WorkspaceHome.presentation';
import { useStacks } from './hooks/useStacks';

import type { FlashbarProps } from '@cloudscape-design/components';
import type { StackInfo } from '../../../../invoke/Stack';
import type { WorkspaceLayoutContext } from '../../Layout';

export const WorkspaceHome = (): JSX.Element => {
  const navigate = useNavigate();
  const workspace = useOutletContext<WorkspaceLayoutContext>();
  const { loadSideMenu } = useWorkspaceResourceContext();
  const [flashbarItems, setFlashbarItems] = useState<FlashbarProps.MessageDefinition[]>([]);
  const { importStack, loadStacks, deleteStack, stacks, state } = useStacks();
  const [selectedItems, setSelectedItems] = useState<StackInfo[]>([]);
  const tableCollection = useCollection<StackInfo>(stacks, {
    pagination: { pageSize: 10 },
  });

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
          await loadStacks();
          await loadSideMenu();
        })
        .catch((error) => {
          const id = uuidV4();
          setFlashbarItems([
            ...flashbarItems,
            {
              type: 'error',
              header: '新規スタックのインポートに失敗しました',
              content: typeof error === 'string' ? error : undefined,
              dismissible: true,
              dismissLabel: 'close',
              id: id,
              onDismiss: () => {
                setFlashbarItems((items) => items.filter((e) => e.id !== id));
              },
            },
          ]);
        });
    }
  }, [workspace, flashbarItems, importStack, loadStacks, loadSideMenu]);

  // biome-ignore lint/correctness/useExhaustiveDependencies: <explanation>
  useEffect(() => {
    loadStacks().catch((error) => {
      const id = uuidV4();
      setFlashbarItems([
        ...flashbarItems,
        {
          type: 'error',
          header: 'スタックの読み込みに失敗しました',
          content: typeof error === 'string' ? error : undefined,
          dismissible: true,
          dismissLabel: 'close',
          id: id,
          onDismiss: () => {
            setFlashbarItems((items) => items.filter((e) => e.id !== id));
          },
        },
      ]);
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
    deleteStack(selectedItems[0]);
    setSelectedItems([]);
    await loadSideMenu();
  };

  return (
    <WorkspaceHomePresentation
      selectedItems={selectedItems}
      isLoading={state === 'loading'}
      onSelectionChange={({ detail }) => setSelectedItems(detail.selectedItems)}
      onClickReloadStack={loadStacks}
      tableCollection={tableCollection}
      onClickStackName={onClickStackName}
      onClickImportStack={importStackWrap}
      onClickDeleteStack={onClickDeleteStack}
      onClickWorkspaceEdit={() => navigate(`/workspaces/${workspace.id}/edit`)}
    />
  );
};
