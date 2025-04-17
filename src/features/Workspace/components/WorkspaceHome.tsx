import { useEffect, useState } from 'react';
import { useNavigate, useOutletContext } from 'react-router';
import { v4 as uuidv4 } from 'uuid';

import { useCollection } from '@cloudscape-design/collection-hooks';
import Box from '@cloudscape-design/components/box';
import Button from '@cloudscape-design/components/button';
import ContentLayout from '@cloudscape-design/components/content-layout';
import Flashbar from '@cloudscape-design/components/flashbar';
import Header from '@cloudscape-design/components/header';
import Pagination from '@cloudscape-design/components/pagination';
import SpaceBetween from '@cloudscape-design/components/space-between';
import Table from '@cloudscape-design/components/table';

import { loadStacks } from '../../../invoke/Stack';

import type { FlashbarProps } from '@cloudscape-design/components';
import type { WorkspaceLayoutLoaderData } from '../Loader';
export interface Stack {
  name: string;
  description: string;
}

export const WorkspaceHome = (): JSX.Element => {
  const workspace = useOutletContext<WorkspaceLayoutLoaderData>();
  const navigate = useNavigate();
  const [flashbarItems, setFlashbarItems] = useState<
    FlashbarProps.MessageDefinition[]
  >([]);
  console.log('Component', workspace);
  const [stacks, setStacks] = useState<Stack[]>([]);

  // biome-ignore lint/correctness/useExhaustiveDependencies: <explanation>
  useEffect(() => {
    loadStacks(workspace.directory)
      .then((stacks) => {
        setStacks(stacks);
        console.log('Stacks', stacks);
      })
      .catch((error) => {
        const id = uuidv4();
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
              setFlashbarItems(flashbarItems.filter((e) => e.id !== id));
            },
          },
        ]);
      });
  }, []);

  const { items, collectionProps, paginationProps } = useCollection<{
    name: string;
    description: string;
  }>([], {
    pagination: { pageSize: 10 },
  });
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
                <Button
                  variant="normal"
                  onClick={() => navigate(`/workspaces/${workspace.id}/edit`)}
                >
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
            cell: (e) => e.name,
            isRowHeader: true,
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
                <Button variant="primary">インポートスタック</Button>
              </SpaceBetween>
            }
          >
            スタック一覧
          </Header>
        }
        pagination={<Pagination {...paginationProps} />}
      />
    </ContentLayout>
  );
};
