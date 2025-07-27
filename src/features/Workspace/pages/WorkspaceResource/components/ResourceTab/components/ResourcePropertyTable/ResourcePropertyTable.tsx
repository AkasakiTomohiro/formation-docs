import { useEffect, useState } from 'react';

import { getCloudFormationSchema } from '../../../../../../../../invoke/CloudFormationSchema';
import { createResourceTableItems } from '../../../../lib/CreateResourceTableItems';
import { ResourcePropertyTablePresentation } from './ResourcePropertyTable.presentation';
import { getStackResourceProperties } from './lib/GetStackResourceProperties';
import { getStackResourcePropertiesReasons } from './lib/GetStackResourcePropertiesReasons';
import { updateStackMeta } from './lib/UpdateStackMeta';

import type { ResourcePropertyTablePresentationProps } from './ResourcePropertyTable.presentation';

import type { ResourceTableItem } from '../../../../lib/CreateResourceTableItems';
import type { CloudFormationSchema } from '../../../types/CloudFormationSchema';
export type ResourcePropertyTableProps = {
  /**
   * スタックID
   */
  stackId: string;

  /**
   * サービス名
   */
  serviceName: string;

  /**
   * リソース名
   */
  resourceName: string;

  /**
   * 選択している論理ID
   */
  selectedLogicalId: string;
};

export const ResourcePropertyTable = ({
  stackId,
  serviceName,
  resourceName,
  selectedLogicalId,
}: ResourcePropertyTableProps): JSX.Element => {
  // ネストされたプロパティの展開状態を管理するためのステート
  // biome-ignore lint/suspicious/noExplicitAny: <explanation>
  const [expandedItems, setExpandedItems] = useState<any>();

  // 編集モードの状態を管理するためのステート
  const [isEdit, setIsEdit] = useState(false);

  // CloudFormationのスキーマを管理するためのステート
  const [schema, setSchema] = useState<CloudFormationSchema | undefined>(undefined);

  // ファイルに保存されている理由を管理するためのステート
  const [reasons, setReasons] = useState<Record<string, string>>({});

  // 編集中の理由を管理するためのステート
  const [editingReasons, setEditingReasons] = useState<Record<string, string>>({});

  // 選択した論理IDがもつプロパティを管理するためのステート
  const [properties, setProperties] = useState<ResourceTableItem[]>([]);

  // テーブルの表示カラムの設定を管理するためのステート
  const [preferences, setPreferences] = useState({
    contentDisplay: [
      { id: 'property', visible: true },
      { id: 'type', visible: true },
      { id: 'description', visible: true },
      { id: 'value', visible: true },
      { id: 'reason', visible: true },
    ],
  });

  useEffect(() => {
    Promise.all([
      getCloudFormationSchema(serviceName, resourceName),
      getStackResourceProperties({
        stack_id: stackId,
        logical_id: selectedLogicalId,
      }),
      getStackResourcePropertiesReasons({
        stack_id: stackId,
        logical_id: selectedLogicalId,
      }),
    ]).then(([schemaStr, properties, reasons]) => {
      console.log('properties', properties);
      console.log('reasons', reasons);
      const schemaParsed = JSON.parse(schemaStr) as CloudFormationSchema;
      setSchema(schemaParsed);
      const resourceTableItems = createResourceTableItems(schemaParsed, properties);
      setProperties(resourceTableItems);
      setExpandedItems(createExpandedItems(resourceTableItems));
      setReasons(reasons);
      setIsEdit(false);
    });
  }, [stackId, serviceName, resourceName, selectedLogicalId]);

  const onSave = async () => {
    setReasons(editingReasons);
    setIsEdit(false);
    await updateStackMeta({
      stack_id: stackId,
      logical_id: selectedLogicalId as string,
      reasons: editingReasons,
    });

    getStackResourceProperties({
      stack_id: stackId,
      logical_id: selectedLogicalId as string,
    }).then((properties) => {
      const resourceTableItems = createResourceTableItems(schema as CloudFormationSchema, properties);
      setProperties(resourceTableItems);
    });
  };

  const onCancel = () => {
    setEditingReasons(reasons);
    setIsEdit(false);
  };

  const onClickEdit = () => {
    setEditingReasons(reasons);
    setIsEdit(true);
  };

  const onChangeReason: ResourcePropertyTablePresentationProps['onChangeReason'] =
    (item) =>
    ({ detail }) => {
      setEditingReasons((prev) => ({
        ...prev,
        [item.id]: detail.value,
      }));
    };

  return (
    <ResourcePropertyTablePresentation
      properties={properties}
      isEdit={isEdit}
      selectedLogicalId={selectedLogicalId}
      reasons={reasons}
      editingReasons={editingReasons}
      onClickSave={onSave}
      onClickCancel={onCancel}
      onClickEdit={onClickEdit}
      onChangeReason={onChangeReason}
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
      preferences={preferences}
      onConfirmPreferences={({ detail }) =>
        setPreferences({
          contentDisplay: detail.contentDisplay ? [...detail.contentDisplay] : [],
        })
      }
    />
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
