import { useCallback, useEffect, useState } from 'react';
import { useNavigate, useOutletContext } from 'react-router';
import { v4 as uuidV4 } from 'uuid';

import { useCollection } from '@cloudscape-design/collection-hooks';
import { Link } from '@cloudscape-design/components';
import Box from '@cloudscape-design/components/box';
import Button from '@cloudscape-design/components/button';
import ContentLayout from '@cloudscape-design/components/content-layout';
import Flashbar from '@cloudscape-design/components/flashbar';
import Header from '@cloudscape-design/components/header';
import Pagination from '@cloudscape-design/components/pagination';
import SpaceBetween from '@cloudscape-design/components/space-between';
import Table from '@cloudscape-design/components/table';
import { open } from '@tauri-apps/plugin-dialog';

import { useWorkspaceResourceContext } from '../../contexts';
import { useStacks } from './hooks/useStacks';

import type { FlashbarProps } from '@cloudscape-design/components';
import type { StackInfo } from '../../../../invoke/Stack';
import type { WorkspaceLayoutContext } from '../../Layout';
export const WorkspaceHome = (): JSX.Element => {
  const workspace = useOutletContext<WorkspaceLayoutContext>();
  const navigate = useNavigate();
  const [flashbarItems, setFlashbarItems] = useState<FlashbarProps.MessageDefinition[]>([]);
  const { state, stacks, importStack, loadStacks, deleteStack } = useStacks();
  const [selectedItems, setSelectedItems] = useState<StackInfo[]>([]);
  const { items, collectionProps, paginationProps } = useCollection<StackInfo>(stacks, {
    pagination: { pageSize: 10 },
  });
  const { loadSideMenu } = useWorkspaceResourceContext();

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
    console.log('Selected file:', selectedFile);
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
      console.error('Error loading stacks:', error);
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
  return (
    <ContentLayout
      defaultPadding
      header={
        <SpaceBetween size="m">
          <Header
            variant="h1"
            description={workspace.description}
            actions={
              <SpaceBetween size="s">
                <Button variant="normal" onClick={() => navigate(`/workspaces/${workspace.id}/edit`)}>
                  編集
                </Button>
              </SpaceBetween>
            }
          >
            {workspace.name}
          </Header>
          <Flashbar items={flashbarItems} />
        </SpaceBetween>
      }
    >
      <Table
        {...collectionProps}
        columnDefinitions={[
          {
            id: 'Name',
            header: 'Stack name',
            cell: (e) => (
              <Link
                onClick={() =>
                  navigate(`/workspaces/${workspace.id}/resources`, {
                    state: {
                      selectedStackId: e.id,
                      selectedStackName: e.name,
                    },
                  })
                }
              >
                {e.name}
              </Link>
            ),
            isRowHeader: true,
          },
          {
            id: 'DescriptionForMeta',
            header: 'Description for Meta',
            cell: (e) => e.description_from_meta,
          },
          {
            id: 'DescriptionForStack',
            header: 'Description for Stack',
            cell: (e) => e.description_from_stack,
          },
        ]}
        selectionType="single"
        selectedItems={selectedItems}
        onSelectionChange={({ detail }) => setSelectedItems(detail.selectedItems)}
        items={items}
        loadingText="Loading workspace"
        trackBy="name"
        empty={
          <Box margin={{ vertical: 'xs' }} textAlign="center" color="inherit">
            <SpaceBetween size="m">
              <b>No resources</b>
            </SpaceBetween>
          </Box>
        }
        header={
          <Header
            actions={
              <SpaceBetween direction="horizontal" size="xs">
                <Button
                  onClick={async () => {
                    deleteStack(selectedItems[0]);
                    setSelectedItems([]);
                    await loadSideMenu();
                  }}
                  disabled={selectedItems.length === 0}
                >
                  削除
                </Button>
                <Button onClick={loadStacks}>更新</Button>
                <Button variant="primary" onClick={importStackWrap}>
                  インポートスタック
                </Button>
              </SpaceBetween>
            }
          >
            スタック一覧
          </Header>
        }
        pagination={<Pagination {...paginationProps} />}
        loading={state === 'loading'}
      />
    </ContentLayout>
  );
};
