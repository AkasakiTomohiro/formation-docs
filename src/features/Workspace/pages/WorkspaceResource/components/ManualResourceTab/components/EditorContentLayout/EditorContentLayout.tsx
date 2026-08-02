import { useEffect, useState } from 'react';
import { useAppConfigContext } from '../../../../../../../../contexts/AppConfigContext';
import { useFlashbarContext } from '../../../../../../../../contexts/FlashbarContext';
import { getCloudFormationSchema } from '../../../../../../../../invoke/CloudFormationSchema';
import { getTranslation } from '../../../../../../../../invoke/Translation';
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
    description: string;
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
  const { appConfig } = useAppConfigContext();
  const [schema, setSchema] = useState<CloudFormationSchema | undefined>(undefined);
  const [properties, setProperties] = useState<ResourceTableItem[]>([]);
  const [expandedItems, setExpandedItems] = useState<any>();
  const [reasons, setReasons] = useState<Record<string, string>>({});
  const [values, setValues] = useState<string>('');
  const { addFlashbarItem } = useFlashbarContext();
  const [isValid, setIsValid] = useState(true);
  const { viewMode, setViewMode, modifyResourceTab } = useWorkspaceResourceContext();
  // リソースの説明を管理するためのステート
  const [description, setDescription] = useState<string>('');

  // プロパティの説明の翻訳を管理するためのステート
  const [translation, setTranslation] = useState<Record<string, string>>({});

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
      getTranslation({
        lang: appConfig?.language ?? 'En',
        serviceName,
        resourceType: resourceName,
      }),
    ]).then(([cloudformationSchema, properties, resourceMeta, translation]) => {
      console.log('properties', properties);
      console.log('meta', resourceMeta);
      setSchema(cloudformationSchema);
      const resourceTableItems = createResourceTableItems(cloudformationSchema, properties);
      setProperties(resourceTableItems);
      setExpandedItems(createExpandedItems(resourceTableItems));
      setReasons(resourceMeta.reasons);
      setDescription(resourceMeta.description);
      setValues(JSON.stringify(properties, undefined, 2));
      setTranslation(translation);
    });
  }, [selectedResourceId]);

  const onEdit = () => {
    modifyResourceTab(tabId, (originTab: ManualResourceTabInfo) => {
      return {
        ...originTab,
        editingValues: {
          description: description,
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
      reasons: editingValues?.reasons ?? {},
      description: editingValues?.description ?? '',
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
          description: editingValues?.description ?? '',
          reasons: editingValues?.reasons ?? {},
          properties: detail.value,
        },
      };
    });
  };

  const onChangeDescription: EditorContentLayoutPresentationProps['setDescription'] = (description) => {
    modifyResourceTab(tabId, (originTab: ManualResourceTabInfo) => {
      return {
        ...originTab,
        editingValues: {
          description: description,
          reasons: editingValues?.reasons ?? {},
          properties: editingValues?.properties ?? '{}',
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
      editingDescription={editingValues?.description ?? ''}
      setDescription={onChangeDescription}
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
        translation: translation,
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
