import { useEffect, useState } from 'react';
import { useOutletContext } from 'react-router';
import { v4 as uuidV4 } from 'uuid';

import { getCloudFormationSchema } from '../../../../../../../../invoke/CloudFormationSchema';
import {
  getManualManagementResourceProperties,
  getManualManagementResourceReasons,
  updateManualResourceMeta,
  updateManualResourceProperties,
} from '../../../../../../../../invoke/ManualManagementResource';
import { createResourceTableItems } from '../../../../lib/CreateResourceTableItems';
import { createExpandedItems } from '../../../PropertyTable';
import { EditorContentLayoutPresentation } from './EditorContentLayout.presentation';

import type { CodeEditorProps } from '@cloudscape-design/components';
import type { WorkspaceLayoutContext } from '../../../../../../Layout';
import type { ResourceTableItem } from '../../../../lib/CreateResourceTableItems';
import type { CloudFormationSchema } from '../../../types/CloudFormationSchema';
import type { ViewMode } from './EditorContentLayout.presentation';

export type EditorContentLayoutProps = {
  selectedResourceId: string;
  serviceName: string;
  resourceName: string;
};

export const EditorContentLayout = ({ selectedResourceId, serviceName, resourceName }: EditorContentLayoutProps) => {
  const [schema, setSchema] = useState<CloudFormationSchema | undefined>(undefined);
  const [items, setItems] = useState<ResourceTableItem[]>([]);
  // biome-ignore lint/suspicious/noExplicitAny: <explanation>
  const [expandedItems, setExpandedItems] = useState<any>();
  const [isEdit, setIsEdit] = useState(false);
  const [reasons, setReasons] = useState<Record<string, string>>({});
  const [editingReasons, setEditingReasons] = useState<Record<string, string>>({});
  const [mode, setMode] = useState<ViewMode>('reason');
  const [values, setValues] = useState<string>('');
  const [editingValues, setEditingValues] = useState<string>('');
  // biome-ignore lint/suspicious/noExplicitAny: <explanation>
  const [acePreferences, setAcePreferences] = useState<CodeEditorProps.Preferences>({} as any);
  const { setFlashbarItems } = useOutletContext<WorkspaceLayoutContext>();
  const [isValid, setIsValid] = useState(true);

  // biome-ignore lint/correctness/useExhaustiveDependencies: <explanation>
  useEffect(() => {
    if (schema === undefined) {
      return;
    }
    getCloudFormationSchema(serviceName, resourceName).then((schemaStr) => {
      const schema = JSON.parse(schemaStr) as CloudFormationSchema;
      setSchema(JSON.parse(schemaStr));
      Promise.all([
        getManualManagementResourceProperties({
          resource_id: selectedResourceId,
        }),
        getManualManagementResourceReasons({
          resource_id: selectedResourceId,
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
    });
  }, [selectedResourceId, schema]);

  const onEdit = () => {
    setEditingReasons(reasons);
    setEditingValues(values);
    setIsEdit(!isEdit);
  };

  const onCancel = () => {
    setEditingReasons(reasons);
    setIsEdit(!isEdit);
  };

  const onSave = async () => {
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
      resource_id: selectedResourceId as string,
      reasons: editingReasons,
    });
    updateManualResourceProperties({
      resource_id: selectedResourceId as string,
      properties: editingValues,
    });
    const resourceTableItems = createResourceTableItems(schema as CloudFormationSchema, JSON.parse(editingValues));
    setItems(resourceTableItems);
    setExpandedItems(createExpandedItems(resourceTableItems));
    setValues(editingValues);
  };

  return (
    <EditorContentLayoutPresentation
      selectedResourceId={selectedResourceId}
      properties={items}
      isEdit={isEdit}
      viewMode={mode}
      reasons={reasons}
      editingReasons={editingReasons}
      expandedItems={expandedItems}
      values={values}
      editingValues={editingValues}
      acePreferences={acePreferences}
      onClickEdit={onEdit}
      onClickCancel={onCancel}
      onClickSave={onSave}
      setExpandedItems={setExpandedItems}
      setEditingReasons={setEditingReasons}
      onChangeSegmentedControl={({ detail }) => setMode(detail.selectedId as ViewMode)}
      onValidate={({ detail }) => {
        setIsValid(detail.annotations.length === 0);
      }}
      onPreferencesChange={(event) => {
        setAcePreferences(event.detail);
      }}
      onDelayedChange={({ detail }) => setEditingValues(detail.value)}
    />
  );
};
