import { useEffect, useState } from 'react';

import { getCloudFormationSchema } from '../../../../../../../../invoke/CloudFormationSchema';
import { createResourceTableItems } from '../../../../lib/CreateResourceTableItems';
import { createExpandedItems } from '../../../PropertyTable';
import { ResourcePropertyTablePresentation } from './ResourcePropertyTable.presentation';
import { getStackResourceProperties } from './lib/GetStackResourceProperties';
import { getStackResourcePropertiesReasons } from './lib/GetStackResourcePropertiesReasons';
import { loadParameterAndResourceList } from './lib/LoadParameterAndResourceList';
import { updateStackMeta } from './lib/UpdateStackMeta';

import type { ResourceTableItem } from '../../../../lib/CreateResourceTableItems';
import type { CloudFormationSchema } from '../../../types/CloudFormationSchema';
import type { LoadParameterAndResourceListResult } from './lib/LoadParameterAndResourceList';

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

  // ファイルに保存されている理由を管理するためのステート
  const [reasons, setReasons] = useState<Record<string, string>>({});

  // 編集中の理由を管理するためのステート
  const [editingReasons, setEditingReasons] = useState<Record<string, string>>({});

  // 選択した論理IDがもつプロパティを管理するためのステート
  const [properties, setProperties] = useState<ResourceTableItem[]>([]);

  // 組み込み関数用のデータ（ParameterとResourceList）を管理するためのステート
  const [parameterAndResourceList, setParameterAndResourceList] = useState<LoadParameterAndResourceListResult>({
    parameters: [],
    resources: {},
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
      loadParameterAndResourceList({ stack_id: stackId }),
    ]).then(([schemaStr, properties, reasons, parameterAndResourceList]) => {
      console.log('properties', properties);
      console.log('reasons', reasons);
      console.log('parameterAndResourceList', parameterAndResourceList);
      const schemaParsed = JSON.parse(schemaStr) as CloudFormationSchema;
      const resourceTableItems = createResourceTableItems(schemaParsed, properties, parameterAndResourceList);
      setProperties(resourceTableItems);
      setExpandedItems(createExpandedItems(resourceTableItems));
      setReasons(reasons);
      setParameterAndResourceList(parameterAndResourceList);
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

    await Promise.all([
      getCloudFormationSchema(serviceName, resourceName),
      getStackResourceProperties({
        stack_id: stackId,
        logical_id: selectedLogicalId as string,
      }),
    ]).then(([schemaStr, properties]) => {
      const schemaParsed = JSON.parse(schemaStr) as CloudFormationSchema;
      const resourceTableItems = createResourceTableItems(schemaParsed, properties, parameterAndResourceList);
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
