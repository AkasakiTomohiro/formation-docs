import { useEffect, useState } from 'react';

import { getCloudFormationSchema } from '../../../../../../../../invoke/CloudFormationSchema';
import { useWorkspaceResourceContext } from '../../../../../../contexts';
import { createResourceTableItems } from '../../../../lib/CreateResourceTableItems';
import { createExpandedItems } from '../../../PropertyTable';
import { ResourcePropertyTablePresentation } from './ResourcePropertyTable.presentation';
import { getStackResourceProperties } from './lib/GetStackResourceProperties';
import { getStackResourcePropertiesReasons } from './lib/GetStackResourcePropertiesReasons';
import { loadParameterAndResourceList } from './lib/LoadParameterAndResourceList';
import { updateStackMeta } from './lib/UpdateStackMeta';

import type { TemplateSummary } from '../../../../../../contexts/lib/LoadTemplateSummary';
import type { ResourceTableItem } from '../../../../lib/CreateResourceTableItems';
import type { CloudFormationSchema } from '../../../types/CloudFormationSchema';
import type { LoadParameterAndResourceListResult } from './lib/LoadParameterAndResourceList';

export type ResourcePropertyTableProps = {
  /**
   * スタックID
   */
  stackId: string;

  /**
   * スタック名
   */
  stackName: string;

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
  stackName,
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

  const { sideMenu, allStackOutputs } = useWorkspaceResourceContext();

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
      const resourceTableItems = createResourceTableItems(schemaParsed, properties, {
        ...parameterAndResourceList,
        stackId,
        stackName,
        externalResources: createExternalResources(allStackOutputs, sideMenu),
      });
      setProperties(resourceTableItems);
      setExpandedItems(createExpandedItems(resourceTableItems));
      setReasons(reasons);
      setParameterAndResourceList(parameterAndResourceList);
      setIsEdit(false);
    });
  }, [stackId, stackName, serviceName, resourceName, selectedLogicalId, allStackOutputs, sideMenu]);

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
      const resourceTableItems = createResourceTableItems(schemaParsed, properties, {
        ...parameterAndResourceList,
        stackId,
        stackName,
        externalResources: createExternalResources(allStackOutputs, sideMenu),
      });
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

/**
 * スタックのOutputsから外部リソース情報を作成する
 * @param allStackOutputs 全スタックのOutputs
 * @param sideMenu サイドメニュー
 * @returns externalResources
 */
const createExternalResources = (
  allStackOutputs: Record<string, string>,
  sideMenu: TemplateSummary[],
): Record<string, { stackId: string; stackName: string }> => {
  const externalResources: Record<string, { stackId: string; stackName: string }> = {};
  for (const [key, stackId] of Object.entries(allStackOutputs)) {
    const stackName = sideMenu.find((item) => item.id === stackId)?.sectionGroupName ?? '';
    externalResources[key] = { stackId, stackName };
  }
  return externalResources;
};
