import { useEffect, useState } from 'react';
import { getCloudFormationSchema } from '../../../../../../../../invoke/CloudFormationSchema';
import { useWorkspaceResourceContext } from '../../../../../../contexts';
import { createResourceTableItems } from '../../../../lib/CreateResourceTableItems';
import { createExpandedItems } from '../../../PropertyTable';
import { getStackMeta } from './lib/GetStackMeta';
import { getStackResourceProperties } from './lib/GetStackResourceProperties';
import { loadParameterAndResourceList } from './lib/LoadParameterAndResourceList';
import { updateStackMeta } from './lib/UpdateStackMeta';
import { ResourcePropertyTablePresentation } from './ResourcePropertyTable.presentation';
import type { ResourceTabInfo } from '../../../../../../contexts';
import type { TemplateSummary } from '../../../../../../contexts/lib/LoadTemplateSummary';
import type { ResourceTableItem } from '../../../../lib/CreateResourceTableItems';
import type { CloudFormationSchema } from '../../../types/CloudFormationSchema';
import type { LoadParameterAndResourceListResult } from './lib/LoadParameterAndResourceList';

export type ResourcePropertyTableProps = {
  /**
   * タブID
   */
  tabId: string;

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

  /**
   * 編集中の値
   */
  editingValues?: ResourceTabInfo['editingValues'];
};

export const ResourcePropertyTable = ({
  tabId,
  stackId,
  stackName,
  serviceName,
  resourceName,
  selectedLogicalId,
  editingValues,
}: ResourcePropertyTableProps): JSX.Element => {
  // ネストされたプロパティの展開状態を管理するためのステート
  const [expandedItems, setExpandedItems] = useState<any>();

  // ファイルに保存されている理由を管理するためのステート
  const [reasons, setReasons] = useState<Record<string, string>>({});

  // 選択した論理IDがもつプロパティを管理するためのステート
  const [properties, setProperties] = useState<ResourceTableItem[]>([]);

  // 組み込み関数用のデータ（ParameterとResourceList）を管理するためのステート
  const [parameterAndResourceList, setParameterAndResourceList] = useState<LoadParameterAndResourceListResult>({
    parameters: [],
    resources: {},
  });

  // ファイルに保存されているリソースの説明を管理するためのステート
  const [description, setDescription] = useState<string>('');

  const { sideMenu, allStackOutputs, modifyResourceTab } = useWorkspaceResourceContext();

  useEffect(() => {
    Promise.all([
      getCloudFormationSchema(serviceName, resourceName),
      getStackResourceProperties({
        stack_id: stackId,
        logical_id: selectedLogicalId,
      }),
      getStackMeta({
        stack_id: stackId,
        logical_id: selectedLogicalId,
      }),
      loadParameterAndResourceList({ stack_id: stackId }),
    ]).then(([schemaStr, properties, stackMeta, parameterAndResourceList]) => {
      console.log('properties', properties);
      console.log('stackMeta', stackMeta);
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
      setReasons(stackMeta.reasons);
      setDescription(stackMeta.description);
      setParameterAndResourceList(parameterAndResourceList);
    });
  }, [stackId, stackName, serviceName, resourceName, selectedLogicalId, allStackOutputs, sideMenu]);

  const onSave = async () => {
    await updateStackMeta({
      stack_id: stackId,
      logical_id: selectedLogicalId as string,
      reasons: editingValues?.reasons ?? {},
      description: editingValues?.description ?? '',
    });

    // 編集中の値をクリア
    modifyResourceTab(tabId, (originTab: ResourceTabInfo) => {
      return {
        ...originTab,
        editingValues: undefined,
      };
    });

    await Promise.all([
      getCloudFormationSchema(serviceName, resourceName),
      getStackResourceProperties({
        stack_id: stackId,
        logical_id: selectedLogicalId as string,
      }),
      getStackMeta({
        stack_id: stackId,
        logical_id: selectedLogicalId,
      }),
    ]).then(([schemaStr, properties, stackMeta]) => {
      const schemaParsed = JSON.parse(schemaStr) as CloudFormationSchema;
      const resourceTableItems = createResourceTableItems(schemaParsed, properties, {
        ...parameterAndResourceList,
        stackId,
        stackName,
        externalResources: createExternalResources(allStackOutputs, sideMenu),
      });
      setProperties(resourceTableItems);
      setReasons(stackMeta.reasons);
      setDescription(stackMeta.description);
    });
  };

  const onCancel = () => {
    // 編集中の値をクリア
    modifyResourceTab(tabId, (originTab: ResourceTabInfo) => {
      return {
        ...originTab,
        editingValues: undefined,
      };
    });
  };

  const onClickEdit = () => {
    modifyResourceTab(tabId, (originTab: ResourceTabInfo) => {
      return {
        ...originTab,
        editingValues: { reasons, description },
      };
    });
  };

  const onChangeDescription = (description: string) => {
    modifyResourceTab(tabId, (originTab: ResourceTabInfo) => {
      return {
        ...originTab,
        editingValues: {
          reasons: originTab.editingValues?.reasons ?? {},
          description,
        },
      };
    });
  };

  return (
    <ResourcePropertyTablePresentation
      tabId={tabId}
      properties={properties}
      selectedLogicalId={selectedLogicalId}
      reasons={reasons}
      editingValues={editingValues}
      onClickSave={onSave}
      onClickCancel={onCancel}
      onClickEdit={onClickEdit}
      expandedItems={expandedItems}
      setExpandedItems={setExpandedItems}
      description={description}
      setDescription={onChangeDescription}
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
