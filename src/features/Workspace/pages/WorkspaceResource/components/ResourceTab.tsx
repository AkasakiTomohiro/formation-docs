import { useEffect, useState } from 'react';

import {
  Box,
  Button,
  Container,
  ContentLayout,
  Header,
  Link,
  SpaceBetween,
  StatusIndicator,
  Table,
  TextFilter,
} from '@cloudscape-design/components';

import { getCloudFormationSchema } from '../../../../../invoke/CloudFormationSchema';
import { getStackResourceList } from '../../../../../invoke/Stack';
import { createResourceTableItems } from '../lib/CreateResourceTableItems';

import type { CloudFormationSchema } from './types/CloudFormationSchema';

import type { ResourceTableItem } from '../lib/CreateResourceTableItems';
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
  const [items, setItems] = useState<ResourceTableItem[]>([]);
  // biome-ignore lint/suspicious/noExplicitAny: <explanation>
  const [expandedItems, setExpandedItems] = useState<any>();
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

  useEffect(() => {
    if (selectedLogicalId === undefined || schema === undefined) {
      return;
    }
    setItems(createResourceTableItems(schema));
  }, [selectedLogicalId, schema]);

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
          <Container>
            <Table
              renderAriaLive={({ firstIndex, lastIndex, totalItemsCount }) =>
                `Displaying items ${firstIndex} to ${lastIndex} of ${totalItemsCount}`
              }
              renderLoaderPending={() => (
                <Button variant="inline-link" iconName="add-plus">
                  Show more
                </Button>
              )}
              renderLoaderLoading={() => (
                <StatusIndicator type="loading">Loading items</StatusIndicator>
              )}
              renderLoaderError={() => (
                <StatusIndicator type="error">Loading error</StatusIndicator>
              )}
              renderLoaderEmpty={() => <Box>No resources found</Box>}
              expandableRows={{
                getItemChildren: (item) => item.children ?? [],
                isItemExpandable: (item) => Boolean(item.children),
                expandedItems: expandedItems,
                onExpandableItemToggle: ({ detail }) =>
                  setExpandedItems((prev: ResourceTableItem[] | undefined) => {
                    const next = new Set((prev ?? []).map((item) => item.id));
                    detail.expanded
                      ? next.add(detail.item.id)
                      : next.delete(detail.item.id);
                    return [...next].map((id) => ({ id }));
                  }),
              }}
              columnDefinitions={[
                {
                  id: 'property',
                  header: 'Property',
                  cell: (e) => e.property,
                  isRowHeader: true,
                },
                {
                  id: 'type',
                  header: 'Type',
                  cell: (e) => e.type,
                },
                {
                  id: 'description',
                  header: 'Description',
                  cell: (e) => e.description,
                },
                {
                  id: 'value',
                  header: 'Value',
                  cell: (e) => e.value,
                },
              ]}
              enableKeyboardNavigation
              items={items}
              loadingText="Loading resources"
              trackBy="name"
              empty={
                <Box
                  margin={{ vertical: 'xs' }}
                  textAlign="center"
                  color="inherit"
                >
                  <SpaceBetween size="m">
                    <b>No resources</b>
                    <Button>Create resource</Button>
                  </SpaceBetween>
                </Box>
              }
              filter={
                <TextFilter
                  filteringPlaceholder="Find resources"
                  filteringText=""
                  countText="0 matches"
                />
              }
              header={<Header>Table with expandable rows</Header>}
            />
          </Container>
        </ContentLayout>
      )}
    </div>
  );
};
