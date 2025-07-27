import ace from 'ace-builds/src-noconflict/ace';

import 'ace-builds/css/ace.css';
import 'ace-builds/esm-resolver';
import 'ace-builds/src-noconflict/ext-language_tools';
import 'ace-builds/src-noconflict/mode-json';
import 'ace-builds/src-noconflict/snippets/json';
import 'ace-builds/src-noconflict/theme-github';
import jsonWorker from 'ace-builds/src-noconflict/worker-json?url';
import { useEffect, useState } from 'react';
import { useOutletContext } from 'react-router';
import { v4 as uuidV4 } from 'uuid';

import CodeView from '@cloudscape-design/code-view/code-view';
import jsonHighlight from '@cloudscape-design/code-view/highlight/json';
import {
  Box,
  Button,
  CodeEditor,
  CollectionPreferences,
  Container,
  ContentLayout,
  Header,
  Link,
  SegmentedControl,
  SpaceBetween,
  StatusIndicator,
  Table,
  TextFilter,
  Textarea,
} from '@cloudscape-design/components';

import { getCloudFormationSchema } from '../../../../../../invoke/CloudFormationSchema';
import {
  getManualManagementResourceList,
  getManualManagementResourceProperties,
  getManualManagementResourceReasons,
  updateManualResourceMeta,
  updateManualResourceProperties,
} from '../../../../../../invoke/ManualManagementResource';
import { useWorkspaceResourceContext } from '../../../../contexts';
import { createResourceTableItems } from '../../lib/CreateResourceTableItems';

import type { ManualResourceTabAttr, ManualResourceTabInfo } from '../../../../contexts';

import type { Dispatch } from 'react';
import type { ManualManagementResource } from '../../../../../../invoke/ManualManagementResource';
import type { WorkspaceLayoutContext } from '../../../../Layout';
import type { CloudFormationSchema } from '../types/CloudFormationSchema';

import type { ResourceTableItem } from '../../lib/CreateResourceTableItems';

export type ManualResourceTabProps = ManualResourceTabAttr;

export type BuildManualResourceTabNameProps = {
  serviceName: string;
  resourceName: string;
};
export function buildManualResourceTabName({ serviceName, resourceName }: BuildManualResourceTabNameProps): string {
  return `手動管理リソース - ${serviceName}::${resourceName}`;
}

ace.config.setModuleUrl('ace/mode/json_worker', jsonWorker);

const i18nStrings = {
  loadingState: 'Loading code editor',
  errorState: 'There was an error loading the code editor.',
  errorStateRecovery: 'Retry',

  editorGroupAriaLabel: 'Code editor',
  statusBarGroupAriaLabel: 'Status bar',

  cursorPosition: (row: number, column: number) => `Ln ${row}, Col ${column}`,
  errorsTab: 'Errors',
  warningsTab: 'Warnings',
  preferencesButtonAriaLabel: 'Preferences',

  paneCloseButtonAriaLabel: 'Close',

  preferencesModalHeader: 'Preferences',
  preferencesModalCancel: 'Cancel',
  preferencesModalConfirm: 'Confirm',
  preferencesModalWrapLines: 'Wrap lines',
  preferencesModalTheme: 'Theme',
  preferencesModalLightThemes: 'Light themes',
  preferencesModalDarkThemes: 'Dark themes',
};

const selectModeControl = (
  mode: 'reason' | 'value',
  setMode: Dispatch<React.SetStateAction<'value' | 'reason'>>,
): JSX.Element => (
  <SegmentedControl
    selectedId={mode}
    onChange={({ detail }) => setMode(detail.selectedId as 'reason' | 'value')}
    label="mode select control"
    options={[
      { text: 'Reason', id: 'reason' },
      { text: 'Value', id: 'value' },
    ]}
  />
);

export const ManualResourceTab = (props: ManualResourceTabProps): JSX.Element => {
  const [isLoading, setIsLoading] = useState(true);
  const [schema, setSchema] = useState<CloudFormationSchema | undefined>(undefined);
  const [manualResourceList, setManualResourceList] = useState<ManualManagementResource[]>([]);
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
  const [editingReasons, setEditingReasons] = useState<Record<string, string>>({});
  const [mode, setMode] = useState<'reason' | 'value'>('reason');
  const [values, setValues] = useState<string>('');
  const [editingValues, setEditingValues] = useState<string>('');
  const [acePreferences, setAcePreferences] = useState({});
  const { setFlashbarItems } = useOutletContext<WorkspaceLayoutContext>();
  const { modifyResourceTab } = useWorkspaceResourceContext();
  const [isValid, setIsValid] = useState(true);

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
      setValues(JSON.stringify(properties, undefined, 2));
      setEditingReasons(reasons);
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
                      modifyResourceTab(props.tabId, (originTab: ManualResourceTabInfo) => {
                        return {
                          ...originTab,
                          selectedResourceId: item.resourceId,
                        };
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
          <Button iconName="angle-right" variant="icon" onClick={() => setIsOpen(!isOpen)} />
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
                        setEditingValues(values);
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
                        if (!isValid) {
                          const id = uuidV4();
                          setFlashbarItems((items) => [
                            ...items,
                            {
                              type: 'error',
                              header: '保存に失敗しました',
                              content: 'リソースプロパティのエラーをすべて修正してください',
                              dismissible: true,
                              dismissLabel: 'close',
                              id: id,
                              onDismiss: () => {
                                setFlashbarItems((items) => items.filter((e) => e.id !== id));
                              },
                            },
                          ]);
                          return;
                        }
                        setReasons(editingReasons);
                        setIsEdit(!isEdit);
                        updateManualResourceMeta({
                          resource_id: props.selectedResourceId as string,
                          reasons: editingReasons,
                        });
                        updateManualResourceProperties({
                          resource_id: props.selectedResourceId as string,
                          properties: editingValues,
                        });
                        const resourceTableItems = createResourceTableItems(
                          schema as CloudFormationSchema,
                          JSON.parse(editingValues),
                        );
                        setItems(resourceTableItems);
                        setExpandedItems(createExpandedItems(resourceTableItems));
                        setValues(editingValues);
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
          {mode === 'reason' && (
            <Table
              renderAriaLive={({ firstIndex, lastIndex, totalItemsCount }) =>
                `Displaying items ${firstIndex} to ${lastIndex} of ${totalItemsCount}`
              }
              renderLoaderPending={() => (
                <Button variant="inline-link" iconName="add-plus">
                  Show more
                </Button>
              )}
              renderLoaderLoading={() => <StatusIndicator type="loading">Loading items</StatusIndicator>}
              renderLoaderError={() => <StatusIndicator type="error">Loading error</StatusIndicator>}
              renderLoaderEmpty={() => <Box>No resources found</Box>}
              expandableRows={{
                getItemChildren: (item) => item.children ?? [],
                isItemExpandable: (item) => Boolean(item.children),
                expandedItems: expandedItems,
                onExpandableItemToggle: ({ detail }) =>
                  setExpandedItems((prev: ResourceTableItem[] | undefined) => {
                    const next = new Set((prev ?? []).map((item) => item.id));
                    detail.expanded ? next.add(detail.item.id) : next.delete(detail.item.id);
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
                  cell: (e) => <div style={{ whiteSpace: 'pre-line' }}>{e.type}</div>,
                  width: 150,
                  minWidth: 100,
                },
                {
                  id: 'description',
                  header: 'Description',
                  cell: (e) => <div style={{ whiteSpace: 'pre-line' }}>{e.description}</div>,
                  width: 500,
                },
                {
                  id: 'value',
                  header: 'Value',
                  cell: (e) => {
                    return <div style={{ whiteSpace: 'pre-line' }}>{e.value === undefined ? '' : `${e.value}`}</div>;
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
                    return <div style={{ whiteSpace: 'pre-line' }}>{reasons[e.id]}</div>;
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
                <Box margin={{ vertical: 'xs' }} textAlign="center" color="inherit">
                  <SpaceBetween size="m">
                    <b>No resources</b>
                    <Button>Create resource</Button>
                  </SpaceBetween>
                </Box>
              }
              filter={<TextFilter filteringPlaceholder="Find resources" filteringText="" countText="0 matches" />}
              header={<Header actions={selectModeControl(mode, setMode)} />}
              preferences={
                <CollectionPreferences
                  title="Preferences"
                  confirmLabel="Confirm"
                  cancelLabel="Cancel"
                  preferences={preferences}
                  onConfirm={({ detail }) =>
                    setPreferences({
                      contentDisplay: detail.contentDisplay ? [...detail.contentDisplay] : [],
                    })
                  }
                  contentDisplayPreference={{
                    description: 'Customize the visibility and order of the columns.',
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
          )}
          {mode === 'value' && (
            <Container header={<Header variant="h2" actions={selectModeControl(mode, setMode)} />}>
              {isEdit ? (
                <CodeEditor
                  ace={ace}
                  language="json"
                  themes={{ light: ['github'], dark: ['github'] }}
                  value={editingValues}
                  // loading={loading}
                  onValidate={({ detail }) => {
                    setIsValid(detail.annotations.length === 0);
                  }}
                  preferences={acePreferences}
                  onPreferencesChange={(event) => {
                    setAcePreferences(event.detail);
                  }}
                  onDelayedChange={({ detail }) => setEditingValues(detail.value)}
                  i18nStrings={i18nStrings}
                />
              ) : (
                <CodeView content={values} lineNumbers highlight={jsonHighlight} />
              )}
            </Container>
          )}
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
