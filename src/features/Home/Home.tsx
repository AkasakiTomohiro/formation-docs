import { useCollection } from '@cloudscape-design/collection-hooks';
import Box from '@cloudscape-design/components/box';
import Button from '@cloudscape-design/components/button';
import ContentLayout from '@cloudscape-design/components/content-layout';
import Header from '@cloudscape-design/components/header';
import Pagination from '@cloudscape-design/components/pagination';
import SpaceBetween from '@cloudscape-design/components/space-between';
import Table from '@cloudscape-design/components/table';
import { open } from '@tauri-apps/plugin-dialog';
import { useCallback } from 'react';
import { useWorkspace } from '../../hooks/useWorkspace';

export const Home = (): JSX.Element => {
  const { state, workspaces, addWorkspace, loadWorkspaces } = useWorkspace();
  const { items, collectionProps, paginationProps } = useCollection(
    workspaces.workspaces,
    {
      pagination: { pageSize: 10 },
    },
  );

  const openWorkspace = useCallback(async () => {
    const selected = await open({
      multiple: false,
      directory: true,
      defaultPath: '~/Desktop',
    });
    if (selected != null) {
      const result = await addWorkspace({ directory: selected });
      console.log({ result });
    }
  }, [addWorkspace]);

  return (
    <ContentLayout
      defaultPadding
      header={
        <SpaceBetween size="m">
          <Header variant="h1">Home</Header>
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
