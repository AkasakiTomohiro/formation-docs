import { useState } from 'react';

import { useCollection } from '@cloudscape-design/collection-hooks';
import {
  Box,
  Button,
  ContentLayout,
  Header,
  Pagination,
  SpaceBetween,
  Table,
} from '@cloudscape-design/components';

export type ManualManagementTabProps = {
  stackId: 'manualManagement';
  sectionGroupName: string;
};

type ManualManagementResource = {
  serviceName: string;
  resourceName: string;
  resourceId: string;
};

export const ManualManagementTab = (
  props: ManualManagementTabProps,
): JSX.Element => {
  const [resources, setResources] = useState<ManualManagementResource[]>([]);
  const { items, collectionProps, paginationProps } = useCollection(resources, {
    pagination: { pageSize: 10 },
  });

  return (
    <ContentLayout
      header={<Header variant="h1">{props.sectionGroupName}</Header>}
    >
      <Table
        {...collectionProps}
        columnDefinitions={[
          {
            id: 'resourceId',
            header: 'Resource ID',
            cell: (e) => e.resourceId,
            isRowHeader: true,
          },
          {
            id: 'serviceName',
            header: 'Service name',
            cell: (e) => e.serviceName,
          },
          {
            id: 'resourceName',
            header: 'Resource name',
            cell: (e) => e.resourceName,
          },
        ]}
        selectionType="single"
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
                <Button variant="primary" onClick={() => {}}>
                  新規リソース
                </Button>
              </SpaceBetween>
            }
          >
            リソース一覧
          </Header>
        }
        pagination={<Pagination {...paginationProps} />}
      />
    </ContentLayout>
  );
};
