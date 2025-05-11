import { useEffect, useState } from 'react';

import {
  Box,
  Button,
  Container,
  ContentLayout,
  Header,
  Link,
  SpaceBetween,
  Table,
  TextFilter,
} from '@cloudscape-design/components';

import { getCloudFormationSchema } from '../../../../../invoke/CloudFormationSchema';
import { getStackResourceList } from '../../../../../invoke/Stack';

type ResourceTabProps = {
  stackId: string;
  stackName: string;
  serviceName: string;
  resourceName: string;
};

export const ResourceTab = (props: ResourceTabProps): JSX.Element => {
  const [isLoading, setIsLoading] = useState(true);
  const [schema, setSchema] = useState<CloudFormationSchema | undefined>(
    undefined,
  );
  const [resourceList, setResourceList] = useState<string[]>([]);
  const [filterText, setFilterText] = useState<string>('');
  const [isOpen, setIsOpen] = useState(true);
  const [selectedLogicalId, setSelectedLogicalId] = useState<
    string | undefined
  >(undefined);
  console.log('resourceList', resourceList);
  console.log('schema', schema);

  useEffect(() => {
    Promise.all([
      getCloudFormationSchema(props.serviceName, props.resourceName).then(
        (schemaStr) => setSchema(JSON.parse(schemaStr)),
      ),
      getStackResourceList({
        stack_id: props.stackId,
        service_name: props.serviceName,
        resource_name: props.resourceName,
      }).then((list) => setResourceList(list)),
    ]).finally(() => setIsLoading(false));
  }, [props]);

  return (
    <div
      style={{
        width: '100%',
        height: '100%',
        display: 'flex',
      }}
    >
      {isOpen ? (
        <div
          style={{
            width: selectedLogicalId !== undefined ? '300px' : '100%',
            height: '100%',
          }}
        >
          <Table
            columnDefinitions={[
              {
                id: 'logicalId',
                header: 'logical id',
                cell: (item) => (
                  <Link
                    href="#"
                    onClick={(event) => {
                      event.stopPropagation();
                      setSelectedLogicalId(item);
                    }}
                  >
                    {item}
                  </Link>
                ),
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
              <Box
                margin={{ vertical: 'xs' }}
                textAlign="center"
                color="inherit"
              >
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
            header={
              <Header
                actions={
                  selectedLogicalId === undefined ? undefined : (
                    <SpaceBetween direction="horizontal" size="xs">
                      <Button
                        iconName={isOpen ? 'angle-left' : 'angle-right'}
                        variant="icon"
                        onClick={() => setIsOpen(!isOpen)}
                      />
                    </SpaceBetween>
                  )
                }
              >
                リソース
              </Header>
            }
          />
        </div>
      ) : (
        <Container>
          <Button
            iconName="angle-right"
            variant="icon"
            onClick={() => setIsOpen(!isOpen)}
          />
        </Container>
      )}
      {selectedLogicalId !== undefined && (
        <ContentLayout
          defaultPadding
          header={<Header>{selectedLogicalId}</Header>}
        >
          <Container>sample</Container>
        </ContentLayout>
      )}
    </div>
  );
};
