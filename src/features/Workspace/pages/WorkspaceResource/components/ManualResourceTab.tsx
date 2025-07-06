import { useEffect, useState } from 'react';
import { useOutletContext } from 'react-router';

import {
  Box,
  Button,
  CollectionPreferences,
  Container,
  ContentLayout,
  Header,
  Link,
  SpaceBetween,
  StatusIndicator,
  Table,
  TextFilter,
  Textarea,
} from '@cloudscape-design/components';

import { getCloudFormationSchema } from '../../../../../invoke/CloudFormationSchema';
import {
  getManualManagementResourceList,
  getManualManagementResourceProperties,
  getManualManagementResourceReasons,
  updateManualResourceMeta,
} from '../../../../../invoke/ManualManagementResource';
import { createResourceTableItems } from '../lib/CreateResourceTableItems';

import type { ManualManagementResource } from '../../../../../invoke/ManualManagementResource';
import type { WorkspaceLayoutContext } from '../../../Layout';
import type { CloudFormationSchema } from './types/CloudFormationSchema';

import type { ResourceTableItem } from '../lib/CreateResourceTableItems';
export type ManualResourceTabProps = {
  tabId: string;
  serviceName: string;
  resourceName: string;
  selectedResourceId?: string;
};

export type BuildManualResourceTabNameProps = {
  serviceName: string;
  resourceName: string;
};
export function buildManualResourceTabName({
  serviceName,
  resourceName,
}: BuildManualResourceTabNameProps): string {
  return `手動管理リソース - ${serviceName}::${resourceName}`;
}

export const ManualResourceTab = (
  props: ManualResourceTabProps,
): JSX.Element => {
  const [isLoading, setIsLoading] = useState(true);
  const [schema, setSchema] = useState<CloudFormationSchema | undefined>(
    undefined,
  );
  const [manualResourceList, setManualResourceList] = useState<
    ManualManagementResource[]
  >([]);
  const [filterText, setFilterText] = useState<string>('');
  const [isOpen, setIsOpen] = useState(true);
  const [items, setItems] = useState<ResourceTableItem[]>([]);
  // biome-ignore lint/suspicious/noExplicitAny: <explanation>
  const [expandedItems, setExpandedItems] = useState<any>();
  const [preferences, setPreferences] = useState({
    contentDisplay: [
      { id: 'property', visible: true },
      { id: 'type', visible: true },
      { id: 'description', visible: true },
      { id: 'value', visible: true },
      { id: 'reason', visible: true },
    ],
  });
  const [isEdit, setIsEdit] = useState(false);
  const [reasons, setReasons] = useState<Record<string, string>>({});
  const [editingReasons, setEditingReasons] = useState<Record<string, string>>(
    {},
  );
  const { setResourceTabs } = useOutletContext<WorkspaceLayoutContext>();

  useEffect(() => {
    Promise.all([
      getCloudFormationSchema(props.serviceName, props.resourceName),
      getManualManagementResourceList({
        service_name: props.serviceName,
        resource_name: props.resourceName,
      }),
    ])
      .then(([schemaStr, list]) => {
        setSchema(JSON.parse(schemaStr));
        setManualResourceList(list);
      })
      .finally(() => setIsLoading(false));
  }, [props.serviceName, props.resourceName]);

  useEffect(() => {
    if (props.selectedResourceId === undefined || schema === undefined) {
      return;
    }
    Promise.all([
      getManualManagementResourceProperties({
        resource_id: props.selectedResourceId,
      }),
      getManualManagementResourceReasons({
        resource_id: props.selectedResourceId,
      }),
    ]).then(([properties, reasons]) => {
      console.log('properties', properties);
      console.log('reasons', reasons);
      const resourceTableItems = createResourceTableItems(schema, properties);
      setItems(resourceTableItems);
      setExpandedItems(createExpandedItems(resourceTableItems));
      setReasons(reasons);
      setIsEdit(false);
    });
  }, [props.selectedResourceId, schema]);

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
            width: props.selectedResourceId === undefined ? '100%' : '300px',
            minWidth: '300px',
            height: '100%',
          }}
        >
          <Table
            columnDefinitions={[
              {
                id: 'resourceId',
                header: 'resource id',
                cell: (item) => (
                  <Link
                    href="#"
                    onClick={(event) => {
                      event.stopPropagation();
                      setResourceTabs((prev) => {
                        const resourceTabs = prev.map((tab) => {
                          if (tab.tabId === props.tabId) {
                            return {
                              ...tab,
                              selectedResourceId: item.resourceId,
                            };
                          }
                          return tab;
                        });
                        return resourceTabs;
                      });
                    }}
                  >
                    {item.resourceId}
                  </Link>
                ),
                sortingField: 'name',
                isRowHeader: true,
              },
            ]}
            enableKeyboardNavigation
            items={manualResourceList.filter((item) =>
              item.resourceId.toLowerCase().includes(filterText.toLowerCase()),
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
                  props.selectedResourceId === undefined ? undefined : (
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
      {props.selectedResourceId !== undefined && (
        <ContentLayout
          defaultPadding
          header={
            <Header
              actions={
                <SpaceBetween direction="horizontal" size="xs">
                  {!isEdit && (
                    <Button
                      onClick={() => {
                        setEditingReasons(reasons);
                        setIsEdit(!isEdit);
                      }}
                    >
                      編集
                    </Button>
                  )}
                  {isEdit && (
                    <Button
                      onClick={() => {
                        setEditingReasons(reasons);
                        setIsEdit(!isEdit);
                      }}
                    >
                      キャンセル
                    </Button>
                  )}
                  {isEdit && (
                    <Button
                      variant="primary"
                      onClick={async () => {
                        setReasons(editingReasons);
                        setIsEdit(!isEdit);
                        updateManualResourceMeta({
                          resource_id: props.selectedResourceId as string,
                          reasons: editingReasons,
                        });
                      }}
                    >
                      保存
                    </Button>
                  )}
                </SpaceBetween>
              }
            >
              {props.selectedResourceId}
            </Header>
          }
        >
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
            resizableColumns
            columnDefinitions={[
              {
                id: 'property',
                header: 'Property',
                cell: (e) => e.property,
                isRowHeader: true,
                width: 250,
                minWidth: 150,
              },
              {
                id: 'type',
                header: 'Type',
                cell: (e) => (
                  <div style={{ whiteSpace: 'pre-line' }}>{e.type}</div>
                ),
                width: 150,
                minWidth: 100,
              },
              {
                id: 'description',
                header: 'Description',
                cell: (e) => (
                  <div style={{ whiteSpace: 'pre-line' }}>{e.description}</div>
                ),
                width: 500,
              },
              {
                id: 'value',
                header: 'Value',
                cell: (e) => {
                  return (
                    <div style={{ whiteSpace: 'pre-line' }}>
                      {e.value === undefined ? '' : `${e.value}`}
                    </div>
                  );
                },
                width: 300,
                minWidth: 100,
              },
              {
                id: 'reason',
                header: 'Reason',
                cell: (e) => {
                  if (isEdit) {
                    return (
                      <Textarea
                        onChange={({ detail }) =>
                          setEditingReasons((prev) => ({
                            ...prev,
                            [e.id]: detail.value,
                          }))
                        }
                        value={editingReasons[e.id] ?? ''}
                      />
                    );
                  }
                  return (
                    <div style={{ whiteSpace: 'pre-line' }}>
                      {reasons[e.id]}
                    </div>
                  );
                },
                width: 300,
                minWidth: 200,
              },
            ]}
            columnDisplay={preferences.contentDisplay}
            stickyHeader
            enableKeyboardNavigation
            items={items}
            loadingText="Loading resources"
            trackBy="id"
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
            preferences={
              <CollectionPreferences
                title="Preferences"
                confirmLabel="Confirm"
                cancelLabel="Cancel"
                preferences={preferences}
                onConfirm={({ detail }) =>
                  setPreferences({
                    contentDisplay: detail.contentDisplay
                      ? [...detail.contentDisplay]
                      : [],
                  })
                }
                contentDisplayPreference={{
                  description:
                    'Customize the visibility and order of the columns.',
                  options: [
                    {
                      id: 'property',
                      label: 'Property',
                      alwaysVisible: true,
                    },
                    { id: 'type', label: 'Type' },
                    { id: 'description', label: 'Description' },
                    { id: 'value', label: 'Value' },
                    { id: 'reason', label: 'Reason' },
                  ],
                }}
              />
            }
          />
        </ContentLayout>
      )}
    </div>
  );
};

/**
 * リソーステーブル用の展開済みアイテムを作成する
 * @param items
 * @returns
 */
function createExpandedItems(items: ResourceTableItem[]): ResourceTableItem[] {
  const expandedItems: ResourceTableItem[] = [];
  for (const item of items) {
    if (item.children !== undefined) {
      expandedItems.push(item);
      expandedItems.push(...createExpandedItems(item.children));
    }
  }
  return expandedItems;
}
