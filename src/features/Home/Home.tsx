import { useCollection } from '@cloudscape-design/collection-hooks';
import type { FlashbarProps } from '@cloudscape-design/components';
import Box from '@cloudscape-design/components/box';
import Button from '@cloudscape-design/components/button';
import ContentLayout from '@cloudscape-design/components/content-layout';
import Flashbar from '@cloudscape-design/components/flashbar';
import Header from '@cloudscape-design/components/header';
import Pagination from '@cloudscape-design/components/pagination';
import SpaceBetween from '@cloudscape-design/components/space-between';
import Table from '@cloudscape-design/components/table';
import { open } from '@tauri-apps/plugin-dialog';
import { useCallback, useState } from 'react';
import { v4 as uuidv4 } from 'uuid';
import { useWorkspace } from '../../hooks/useWorkspace';

export const Home = (): JSX.Element => {
  const [flashbarItems, setFlashbarItems] = useState<
    FlashbarProps.MessageDefinition[]
  >([]);
  const { state, workspaces, addWorkspace, loadWorkspaces } = useWorkspace();
  const { items, collectionProps, paginationProps } = useCollection(
    workspaces.workspaces,
    {
      pagination: { pageSize: 10 },
    },
  );

  const openWorkspace = useCallback(async () => {
    const selectedDir = await open({
      multiple: false,
      directory: true,
      defaultPath: '~/Desktop',
    });
    if (selectedDir != null) {
      // FIXME: すでに登録されている場合は登録せずただ、ワークスペースを開くだけにする
      // FIXME: ワークスペース登録時のエッジケースを考慮する。たとえば、ディスク容量が足りないや権限がない場合。
      const result = await addWorkspace({ directory: selectedDir });
      if (!result) {
        const id = uuidv4();
        setFlashbarItems([
          ...flashbarItems,
          {
            type: 'error',
            header: '新規Workspaceの読み込みに失敗しました',
            content: `「${selectedDir}」は既に登録されています。`,
            dismissible: true,
            dismissLabel: 'close',
            id: id,
            onDismiss: () => {
              setFlashbarItems(flashbarItems.filter((e) => e.id !== id));
            },
          },
        ]);
      }
    }
  }, [addWorkspace, flashbarItems]);

  return (
    <ContentLayout
      defaultPadding
      header={
        <SpaceBetween size="m">
          <Header variant="h1">Home</Header>
          <Flashbar items={flashbarItems} />
        </SpaceBetween>
      }
    >
      <Table
        {...collectionProps}
        columnDefinitions={[
          {
            id: 'Name',
            header: 'Workspace name',
            cell: (e) => e.name,
            isRowHeader: true,
          },
          {
            id: 'Directory',
            header: 'directory',
            cell: (e) => e.directory,
          },
          {
            id: 'Description',
            header: 'Description',
            cell: (e) => e.description,
          },
        ]}
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
                <Button onClick={loadWorkspaces}>更新</Button>
                <Button variant="primary" onClick={openWorkspace}>
                  新規ワークスペース
                </Button>
              </SpaceBetween>
            }
          >
            ワークスペース一覧
          </Header>
        }
        pagination={<Pagination {...paginationProps} />}
        loading={state === 'loading'}
      />
    </ContentLayout>
  );
};
