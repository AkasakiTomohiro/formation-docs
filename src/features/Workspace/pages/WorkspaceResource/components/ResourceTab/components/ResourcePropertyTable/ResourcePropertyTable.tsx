import { useEffect, useState } from 'react';

import { getCloudFormationSchema } from '../../../../../../../../invoke/CloudFormationSchema';
import { createResourceTableItems } from '../../../../lib/CreateResourceTableItems';
import { createExpandedItems } from '../../../PropertyTable';
import { ResourcePropertyTablePresentation } from './ResourcePropertyTable.presentation';
import { getStackResourceProperties } from './lib/GetStackResourceProperties';
import { getStackResourcePropertiesReasons } from './lib/GetStackResourcePropertiesReasons';
import { updateStackMeta } from './lib/UpdateStackMeta';

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
      expandedItems={expandedItems}
      setExpandedItems={setExpandedItems}
      setEditingReasons={setEditingReasons}
    />
  );
};
