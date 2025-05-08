import { useEffect, useState } from 'react';

import {
  Box,
  Link,
  SpaceBetween,
  Table,
  TextFilter,
} from '@cloudscape-design/components';

import { getStackResourceList } from '../../../../../invoke/Stack';

type ResourceTabProps = {
  stackId: string;
  stackName: string;
  serviceName: string;
  resourceName: string;
};

export const ResourceTab = (props: ResourceTabProps): JSX.Element => {
  const [isLoading, setIsLoading] = useState(true);
  const [resourceList, setResourceList] = useState<string[]>([]);
  const [filterText, setFilterText] = useState<string>('');
  console.log('resourceList', resourceList);

  useEffect(() => {
    getStackResourceList({
      stack_id: props.stackId,
      service_name: props.serviceName,
      resource_name: props.resourceName,
    })
      .then((list) => setResourceList(list))
      .finally(() => setIsLoading(false));
  }, [props]);

  return (
    <Table
      columnDefinitions={[
        {
          id: 'logicalId',
          header: 'logical id',
          cell: (item) => <Link href="#">{item}</Link>,
          sortingField: 'name',
          isRowHeader: true,
        },
      ]}
      enableKeyboardNavigation
      items={resourceList.filter((item) =>
        item.toLowerCase().includes(filterText.toLowerCase()),
      )}
      loadingText="Loading resources"
      loading={isLoading}
      sortingDisabled
      empty={
        <Box margin={{ vertical: 'xs' }} textAlign="center" color="inherit">
          <SpaceBetween size="m">
            <b>No resources</b>
          </SpaceBetween>
        </Box>
      }
      filter={
        <TextFilter
          filteringPlaceholder="Search Resource"
          filteringText={filterText}
          onChange={({ detail }) => setFilterText(detail.filteringText)}
        />
      }
    />
  );
};
