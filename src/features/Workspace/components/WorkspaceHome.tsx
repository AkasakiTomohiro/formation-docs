import { useState } from 'react';
import { useNavigate, useOutletContext } from 'react-router';

import { useCollection } from '@cloudscape-design/collection-hooks';
import Box from '@cloudscape-design/components/box';
import Button from '@cloudscape-design/components/button';
import ContentLayout from '@cloudscape-design/components/content-layout';
import Flashbar from '@cloudscape-design/components/flashbar';
import Header from '@cloudscape-design/components/header';
import Pagination from '@cloudscape-design/components/pagination';
import SpaceBetween from '@cloudscape-design/components/space-between';
import Table from '@cloudscape-design/components/table';

import type { FlashbarProps } from '@cloudscape-design/components';
import type { WorkspaceLayoutLoaderData } from '../Loader';

export const WorkspaceHome = (): JSX.Element => {
  const workspace = useOutletContext<WorkspaceLayoutLoaderData>();
  const navigate = useNavigate();
  const [flashbarItems, setFlashbarItems] = useState<
    FlashbarProps.MessageDefinition[]
  >([]);
  console.log('Component', workspace);
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
