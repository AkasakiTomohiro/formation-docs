import { useEffect, useState } from 'react';
import { useFlashbarContext } from '../../../../../../../../contexts/FlashbarContext';
import { getCloudFormationSchema } from '../../../../../../../../invoke/CloudFormationSchema';
import { useWorkspaceResourceContext } from '../../../../../../contexts';
import { createResourceTableItems } from '../../../../lib/CreateResourceTableItems';
import { createExpandedItems } from '../../../PropertyTable';
import { EditorContentLayoutPresentation } from './EditorContentLayout.presentation';
import { getManualManagementResourceProperties } from './lib/GetManualManagementResourceProperties';
import { getManualManagementResourceMeta } from './lib/getManualManagementResourceMeta';
import { updateManualResourceMeta } from './lib/UpdateManualResourceMeta';
import { updateManualResourceProperties } from './lib/UpdateManualResourceProperties';
import type { ManualResourceTabInfo, ViewMode } from '../../../../../../contexts';
import type { ResourceTableItem } from '../../../../lib/CreateResourceTableItems';
import type { CloudFormationSchema } from '../../../types/CloudFormationSchema';
import type { EditorContentLayoutPresentationProps } from './EditorContentLayout.presentation';

export type EditorContentLayoutProps = {
  tabId: string;
  selectedResourceId: string;
  serviceName: string;
  resourceName: string;
  editingValues?: {
    properties: string;
    reasons: Record<string, string>;
  };
};

export const EditorContentLayout = ({
  tabId,
  selectedResourceId,
  serviceName,
  resourceName,
  editingValues,
}: EditorContentLayoutProps) => {
  const [schema, setSchema] = useState<CloudFormationSchema | undefined>(undefined);
  const [properties, setProperties] = useState<ResourceTableItem[]>([]);
  const [expandedItems, setExpandedItems] = useState<any>();
  const [reasons, setReasons] = useState<Record<string, string>>({});
  const [values, setValues] = useState<string>('');
  const { addFlashbarItem } = useFlashbarContext();
  const [isValid, setIsValid] = useState(true);
  const { viewMode, setViewMode, modifyResourceTab } = useWorkspaceResourceContext();
  // リソースの説明（編集中の値）を管理するためのステート
  const [description, setDescription] = useState<string>('');
  // リソースの説明（ファイルに保存されている値）を管理するためのステート
  const [savedDescription, setSavedDescription] = useState<string>('');

  // biome-ignore lint/correctness/useExhaustiveDependencies: false positive
  useEffect(() => {
    Promise.all([
      getCloudFormationSchema(serviceName, resourceName).then((schemaStr) => JSON.parse(schemaStr)),
      getManualManagementResourceProperties({
        resource_id: selectedResourceId,
      }),
      getManualManagementResourceMeta({
        resource_id: selectedResourceId,
      }),
    ]).then(([cloudformationSchema, properties, resourceMeta]) => {
      console.log('properties', properties);
      console.log('meta', resourceMeta);
      setSchema(cloudformationSchema);
      const resourceTableItems = createResourceTableItems(cloudformationSchema, properties);
      setProperties(resourceTableItems);
      setExpandedItems(createExpandedItems(resourceTableItems));
      setReasons(resourceMeta.reasons);
      setSavedDescription(resourceMeta.description);
      setDescription(resourceMeta.description);
      setValues(JSON.stringify(properties, undefined, 2));
    });
  }, [selectedResourceId]);

  const onEdit = () => {
    modifyResourceTab(tabId, (originTab: ManualResourceTabInfo) => {
      return {
        ...originTab,
        editingValues: {
          reasons: reasons,
          properties: values,
        },
      };
    });
  };

  const onCancel = () => {
    modifyResourceTab(tabId, (originTab: ManualResourceTabInfo) => {
      return {
        ...originTab,
        editingValues: undefined,
      };
    });
    setDescription(savedDescription);
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
    await updateManualResourceMeta({
      resource_id: selectedResourceId as string,
      reasons: editingValues?.reasons,
      description: description,
    });
    await updateManualResourceProperties({
      resource_id: selectedResourceId as string,
      properties: editingValues?.properties ?? '{}',
    });
    const resourceTableItems = createResourceTableItems(
      schema as CloudFormationSchema,
      JSON.parse(editingValues?.properties || '{}'),
    );
    setProperties(resourceTableItems);
    setExpandedItems(createExpandedItems(resourceTableItems));
    modifyResourceTab(tabId, (originTab: ManualResourceTabInfo) => {
      return {
        ...originTab,
        editingValues: undefined,
      };
    });

    await Promise.all([
      getManualManagementResourceProperties({
        resource_id: selectedResourceId,
      }),
      getManualManagementResourceMeta({
        resource_id: selectedResourceId,
      }),
    ]).then(([properties, resourceMeta]) => {
      setReasons(resourceMeta.reasons);
      setSavedDescription(resourceMeta.description);
      setDescription(resourceMeta.description);
      setValues(JSON.stringify(properties, undefined, 2));
    });
  };

  const onPropertiesChange: EditorContentLayoutPresentationProps['resourcePropertyEditorProps']['onDelayedChange'] = ({
    detail,
  }) => {
    modifyResourceTab(tabId, (originTab: ManualResourceTabInfo) => {
      return {
        ...originTab,
        editingValues: {
          reasons: editingValues?.reasons || {},
          properties: detail.value,
        },
      };
    });
  };

  return (
    <EditorContentLayoutPresentation
      tabId={tabId}
      selectedResourceId={selectedResourceId}
      viewMode={viewMode(tabId)}
      description={description}
      setDescription={setDescription}
      onClickEdit={onEdit}
      onClickCancel={onCancel}
      onClickSave={onSave}
      propertyTableProps={{
        tabId: tabId,
        properties: properties,
        reasons: reasons,
        editingReasons: editingValues?.reasons,
        expandedItems: expandedItems,
        setExpandedItems: setExpandedItems,
      }}
      resourcePropertyEditorProps={{
        values: values,
        editingValues: editingValues?.properties,
        onValidate: ({ detail }) => setIsValid(detail.annotations.length === 0),
        onDelayedChange: onPropertiesChange,
      }}
      onChangeSegmentedControl={({ detail }) => setViewMode(tabId, detail.selectedId as ViewMode)}
    />
  );
};
