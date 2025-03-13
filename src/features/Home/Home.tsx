import { useCollection } from '@cloudscape-design/collection-hooks';
import Box from '@cloudscape-design/components/box';
import Button from '@cloudscape-design/components/button';
import ContentLayout from '@cloudscape-design/components/content-layout';
import Header from '@cloudscape-design/components/header';
import Pagination from '@cloudscape-design/components/pagination';
import SpaceBetween from '@cloudscape-design/components/space-between';
import Table from '@cloudscape-design/components/table';
import { useEffect, useRef, useState } from 'react';
import { getWorkspaces } from '../../lib/Workspaces';
import type { Workspaces } from '../../lib/Workspaces';

export const Home = (): JSX.Element => {
  const isFirstRender = useRef(true);
  const [workspaces, setWorkspaces] = useState<Workspaces>({ workspaces: [] });
  const { items, collectionProps, paginationProps } = useCollection(
    [
      {
        name: 'Item 1',
        directory: 'First',
        description: 'This is the first item',
      },
      {
        name: 'Item 2',
        directory: 'Second',
        description: 'This is the second item and it is disabled',
      },
    ],
    {
      pagination: { pageSize: 10 },
    },
  );

  useEffect(() => {
    if (isFirstRender.current) {
      isFirstRender.current = false;
      return;
    }
    getWorkspaces()
      .then((workspaces) => {
        console.log(workspaces);
        setWorkspaces(workspaces);
      })
      .catch((error) => console.log('error', error));
  }, []);

  return (
    <ContentLayout
      defaultPadding
      header={
        <SpaceBetween size="m">
          <Header
            variant="h1"
            actions={
              <SpaceBetween direction="horizontal" size="xs">
                <Button variant="primary">ワークスペースを開く</Button>
              </SpaceBetween>
            }
          >
            Home
          </Header>
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
        header={<Header>ワークスペース一覧</Header>}
        pagination={<Pagination {...paginationProps} />}
      />
    </ContentLayout>
  );
};
