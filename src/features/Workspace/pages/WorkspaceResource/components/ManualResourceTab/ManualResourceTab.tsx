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
  Button,
  CodeEditor,
  Container,
  ContentLayout,
  Header,
  SegmentedControl,
  SpaceBetween,
} from '@cloudscape-design/components';

import { getCloudFormationSchema } from '../../../../../../invoke/CloudFormationSchema';
import {
  getManualManagementResourceProperties,
  getManualManagementResourceReasons,
  updateManualResourceMeta,
  updateManualResourceProperties,
} from '../../../../../../invoke/ManualManagementResource';
import { createResourceTableItems } from '../../lib/CreateResourceTableItems';
import { PropertyTable } from '../PropertyTable';
import { ResourceIdTable } from './components';

import type { ManualResourceTabAttr } from '../../../../contexts';

import type { Dispatch } from 'react';
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
  const [schema, setSchema] = useState<CloudFormationSchema | undefined>(undefined);
  const [items, setItems] = useState<ResourceTableItem[]>([]);
  // biome-ignore lint/suspicious/noExplicitAny: <explanation>
  const [expandedItems, setExpandedItems] = useState<any>();
  const [isEdit, setIsEdit] = useState(false);
  const [reasons, setReasons] = useState<Record<string, string>>({});
  const [editingReasons, setEditingReasons] = useState<Record<string, string>>({});
  const [mode, setMode] = useState<'reason' | 'value'>('reason');
  const [values, setValues] = useState<string>('');
  const [editingValues, setEditingValues] = useState<string>('');
  const [acePreferences, setAcePreferences] = useState({});
  const { setFlashbarItems } = useOutletContext<WorkspaceLayoutContext>();
  const [isValid, setIsValid] = useState(true);

  useEffect(() => {
    getCloudFormationSchema(props.serviceName, props.resourceName).then((schemaStr) => {
      setSchema(JSON.parse(schemaStr));
    });
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
      <ResourceIdTable
        tabId={props.tabId}
        selectedLogicalId={props.selectedResourceId}
        serviceName={props.serviceName}
        resourceName={props.resourceName}
      />
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
            <PropertyTable
              isEdit={isEdit}
              properties={items}
              reasons={reasons}
              expandedItems={expandedItems}
              setExpandedItems={setExpandedItems}
              editingReasons={editingReasons}
              setEditingReasons={setEditingReasons}
              header={<Header actions={selectModeControl(mode, setMode)} />}
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
