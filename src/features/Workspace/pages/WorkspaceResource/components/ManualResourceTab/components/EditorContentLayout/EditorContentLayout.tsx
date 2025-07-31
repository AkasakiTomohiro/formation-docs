import { useEffect, useState } from 'react';
import { useOutletContext } from 'react-router';
import { v4 as uuidV4 } from 'uuid';

import { getCloudFormationSchema } from '../../../../../../../../invoke/CloudFormationSchema';
import { createResourceTableItems } from '../../../../lib/CreateResourceTableItems';
import { createExpandedItems } from '../../../PropertyTable';
import { EditorContentLayoutPresentation } from './EditorContentLayout.presentation';
import { getManualManagementResourceProperties } from './lib/GetManualManagementResourceProperties';
import { getManualManagementResourceReasons } from './lib/GetManualManagementResourceReasons';
import { updateManualResourceMeta } from './lib/UpdateManualResourceMeta';
import { updateManualResourceProperties } from './lib/UpdateManualResourceProperties';

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
  const [properties, setProperties] = useState<ResourceTableItem[]>([]);
  // biome-ignore lint/suspicious/noExplicitAny: <explanation>
  const [expandedItems, setExpandedItems] = useState<any>();
  const [isEdit, setIsEdit] = useState(false);
  const [reasons, setReasons] = useState<Record<string, string>>({});
  const [editingReasons, setEditingReasons] = useState<Record<string, string>>({});
  const [mode, setMode] = useState<ViewMode>('reason');
  const [values, setValues] = useState<string>('');
  const [editingValues, setEditingValues] = useState<string>('');
  const { setFlashbarItems } = useOutletContext<WorkspaceLayoutContext>();
  const [isValid, setIsValid] = useState(true);

  // biome-ignore lint/correctness/useExhaustiveDependencies: <explanation>
  useEffect(() => {
    Promise.all([
      getCloudFormationSchema(serviceName, resourceName).then((schemaStr) => JSON.parse(schemaStr)),
      getManualManagementResourceProperties({
        resource_id: selectedResourceId,
      }),
      getManualManagementResourceReasons({
        resource_id: selectedResourceId,
      }),
    ]).then(([cloudformationSchema, properties, reasons]) => {
      console.log('properties', properties);
      console.log('reasons', reasons);
      setSchema(cloudformationSchema);
      const resourceTableItems = createResourceTableItems(cloudformationSchema, properties);
      setProperties(resourceTableItems);
      setExpandedItems(createExpandedItems(resourceTableItems));
      setReasons(reasons);
      setValues(JSON.stringify(properties, undefined, 2));
      setEditingReasons(reasons);
      setIsEdit(false);
    });
  }, [selectedResourceId]);

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
    setProperties(resourceTableItems);
    setExpandedItems(createExpandedItems(resourceTableItems));
    setValues(editingValues);
  };

  return (
    <EditorContentLayoutPresentation
      selectedResourceId={selectedResourceId}
      isEdit={isEdit}
      viewMode={mode}
      onClickEdit={onEdit}
      onClickCancel={onCancel}
      onClickSave={onSave}
      propertyTableProps={{
        properties: properties,
        reasons: reasons,
        editingReasons: editingReasons,
        expandedItems: expandedItems,
        setExpandedItems: setExpandedItems,
        setEditingReasons: setEditingReasons,
      }}
      resourcePropertyEditorProps={{
        values: values,
        editingValues: editingValues,
        onValidate: ({ detail }) => setIsValid(detail.annotations.length === 0),
        onDelayedChange: ({ detail }) => setEditingValues(detail.value),
      }}
      onChangeSegmentedControl={({ detail }) => setMode(detail.selectedId as ViewMode)}
    />
  );
};
