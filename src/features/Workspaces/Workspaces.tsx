import { useCallback, useEffect, useRef, useState } from 'react';
import { v4 as uuidv4 } from 'uuid';

import { useCollection } from '@cloudscape-design/collection-hooks';
import Box from '@cloudscape-design/components/box';
import Button from '@cloudscape-design/components/button';
import ContentLayout from '@cloudscape-design/components/content-layout';
import Flashbar from '@cloudscape-design/components/flashbar';
import Header from '@cloudscape-design/components/header';
import Link from '@cloudscape-design/components/link';
import Pagination from '@cloudscape-design/components/pagination';
import SpaceBetween from '@cloudscape-design/components/space-between';
import Table from '@cloudscape-design/components/table';
import { invoke } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-dialog';

import { useWorkspaces } from '../../hooks/useWorkspaces';

import type { FlashbarProps } from '@cloudscape-design/components';
import type { WorkspaceExpand } from '../../invoke/Workspace';

export const Workspaces = (): JSX.Element => {
  const isFirstRender = useRef(true);
  const [flashbarItems, setFlashbarItems] = useState<
    FlashbarProps.MessageDefinition[]
  >([]);
  const {
    state,
    workspaces,
    createWorkspace,
    loadWorkspaces,
    deleteWorkspace,
  } = useWorkspaces();
  const { items, collectionProps, paginationProps } = useCollection(
    workspaces,
    {
      pagination: { pageSize: 10 },
    },
  );
  const [selectedItems, setSelectedItems] = useState<WorkspaceExpand[]>([]);

  const openWorkspace = useCallback(
    async (workspace: WorkspaceExpand) => {
      const result = await invoke('open_workspace', {
        id: workspace.id,
        name: workspace.name,
        directory: workspace.directory,
      });
      console.log({ result, workspace });
      if (!result) {
        const id = uuidv4();
        setFlashbarItems([
          ...flashbarItems,
          {
            type: 'error',
            header: 'Workspaceが開けませんでした',
            content: `「${workspace.directory}」が存在することを確認してください`,
            dismissible: true,
            dismissLabel: 'close',
            id: id,
            onDismiss: () => {
              setFlashbarItems(flashbarItems.filter((e) => e.id !== id));
            },
          },
        ]);
      }
    },
    [flashbarItems],
  );

  const newWorkspace = useCallback(async () => {
    const selectedDir = await open({
      multiple: false,
      directory: true,
      defaultPath: '~/Desktop',
    });
    if (selectedDir != null) {
      createWorkspace({ directory: selectedDir })
        .then((workspace) => openWorkspace(workspace))
        .catch((error) => {
          const id = uuidv4();
          setFlashbarItems([
            ...flashbarItems,
            {
              type: 'error',
              header: '新規Workspaceの読み込みに失敗しました',
              content: typeof error === 'string' ? error : undefined,
              dismissible: true,
              dismissLabel: 'close',
              id: id,
              onDismiss: () => {
                setFlashbarItems(flashbarItems.filter((e) => e.id !== id));
              },
            },
          ]);
        });
    }
  }, [createWorkspace, openWorkspace, flashbarItems]);

  // biome-ignore lint/correctness/useExhaustiveDependencies: <explanation>
  useEffect(() => {
    if (isFirstRender.current) {
      isFirstRender.current = false;
      return;
    }
    loadWorkspaces().catch((error) => {
      const id = uuidv4();
      setFlashbarItems([
        ...flashbarItems,
        {
          type: 'error',
          header: 'Workspaceの読み込みに失敗しました',
          content: typeof error === 'string' ? error : undefined,
          dismissible: true,
          dismissLabel: 'close',
          id: id,
          onDismiss: () => {
            setFlashbarItems(flashbarItems.filter((e) => e.id !== id));
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
            cell: (e) => <Link onClick={() => openWorkspace(e)}>{e.name}</Link>,
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
        selectionType="single"
        selectedItems={selectedItems}
        onSelectionChange={({ detail }) =>
          setSelectedItems(detail.selectedItems)
        }
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
                  onClick={() => {
                    deleteWorkspace(selectedItems[0]);
                    setSelectedItems([]);
                  }}
                  disabled={selectedItems.length === 0}
                >
                  削除
                </Button>
                <Button onClick={loadWorkspaces}>更新</Button>
                <Button variant="primary" onClick={newWorkspace}>
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
