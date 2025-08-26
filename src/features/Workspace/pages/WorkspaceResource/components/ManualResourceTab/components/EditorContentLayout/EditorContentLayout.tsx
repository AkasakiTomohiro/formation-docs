import { useEffect, useState } from 'react';

import { useFlashbarContext } from '../../../../../../../../contexts/FlashbarContext';
import { getCloudFormationSchema } from '../../../../../../../../invoke/CloudFormationSchema';
import { createResourceTableItems } from '../../../../lib/CreateResourceTableItems';
import { createExpandedItems } from '../../../PropertyTable';
import { EditorContentLayoutPresentation } from './EditorContentLayout.presentation';
import { getManualManagementResourceProperties } from './lib/GetManualManagementResourceProperties';
import { getManualManagementResourceReasons } from './lib/GetManualManagementResourceReasons';
import { updateManualResourceMeta } from './lib/UpdateManualResourceMeta';
import { updateManualResourcePropertiesAndDescription } from './lib/UpdateManualResourceProperties';

import type { ResourceTableItem } from '../../../../lib/CreateResourceTableItems';
import type { CloudFormationSchema } from '../../../types/CloudFormationSchema';
import type { ViewMode } from './components';

export type EditorContentLayoutProps = {
  selectedResourceId: string;
  serviceName: string;
  resourceName: string;
  description: string; // TODO: このdescriptionは削除
};

export const EditorContentLayout = ({
  selectedResourceId,
  serviceName,
  resourceName,
  description,
}: EditorContentLayoutProps) => {
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
  const { addFlashbarItem } = useFlashbarContext();
  const [isValid, setIsValid] = useState(true);
  const [resourceDescription, setResourceDescription] = useState<string>('');
  const [editingResourceDescription, setEditingResourceDescription] = useState<string>('');

  // biome-ignore lint/correctness/useExhaustiveDependencies: <explanation>
  useEffect(() => {
    // TODO: descriptionの取得処理を追加
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
      setResourceDescription(description);
      setIsEdit(false);
    });
  }, [selectedResourceId]);

  const onEdit = () => {
    setEditingReasons(reasons);
    setEditingValues(values);
    setEditingResourceDescription(resourceDescription);
    setIsEdit(!isEdit);
  };

  const onCancel = () => {
    setEditingReasons(reasons);
    setEditingResourceDescription(resourceDescription);
    setMode('reason');
    setIsEdit(!isEdit);
  };

  const onSave = async () => {
    if (!isValid) {
      addFlashbarItem({
        type: 'error',
        header: '保存に失敗しました',
        content: 'リソースプロパティのエラーをすべて修正してください',
      });
      return;
    }
    setReasons(editingReasons);
    setResourceDescription(editingResourceDescription);
    setMode('reason');
    setIsEdit(!isEdit);
    updateManualResourceMeta({
      resource_id: selectedResourceId as string,
      reasons: editingReasons,
    });
    updateManualResourcePropertiesAndDescription({
      resource_id: selectedResourceId as string,
      properties: editingValues,
      description: editingResourceDescription,
    });
    const resourceTableItems = createResourceTableItems(schema as CloudFormationSchema, JSON.parse(editingValues));
    setProperties(resourceTableItems);
    setExpandedItems(createExpandedItems(resourceTableItems));
    setValues(editingValues);
  };

  return (
    <EditorContentLayoutPresentation
      selectedResourceId={selectedResourceId}
      description={editingResourceDescription}
      isEdit={isEdit}
      viewMode={mode}
      onClickEdit={onEdit}
      onClickCancel={onCancel}
      onClickSave={onSave}
      resourceEditContentProps={{
        setDescription: setEditingResourceDescription,
      }}
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
