import { useCollection } from '@cloudscape-design/collection-hooks';
import Box from '@cloudscape-design/components/box';
import Button from '@cloudscape-design/components/button';
import ContentLayout from '@cloudscape-design/components/content-layout';
import Header from '@cloudscape-design/components/header';
import Pagination from '@cloudscape-design/components/pagination';
import SpaceBetween from '@cloudscape-design/components/space-between';
import Table from '@cloudscape-design/components/table';
import { useParams } from 'react-router';

export const Workspace = (): JSX.Element => {
  const { workspaceId } = useParams();
  const { items, collectionProps, paginationProps } = useCollection([], {
    pagination: { pageSize: 10 },
  });
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
            ワークスペース一覧
          </Header>
        }
        pagination={<Pagination {...paginationProps} />}
      />
    </ContentLayout>
  );
};
