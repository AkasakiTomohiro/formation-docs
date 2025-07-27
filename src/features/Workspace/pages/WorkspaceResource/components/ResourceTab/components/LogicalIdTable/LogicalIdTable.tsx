import { useEffect, useState } from 'react';

import { useWorkspaceResourceContext } from '../../../../../../contexts';
import { LogicalIdTablePresentation } from './LogicalIdTable.presentation';
import { getStackResourceList } from './lib/GetStackResourceList';

import type { ResourceTabInfo } from '../../../../../../contexts';

export type LogicalIdTableProps = {
  /**
   * スタックのタブID
   */
  tabId: string;

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
   * 選択されている論理ID
   */
  selectedLogicalId: string | undefined;
};

export const LogicalIdTable = ({
  tabId,
  stackId,
  serviceName,
  resourceName,
  selectedLogicalId,
}: LogicalIdTableProps): JSX.Element => {
  const { modifyResourceTab } = useWorkspaceResourceContext();
  const [isLoading, setIsLoading] = useState(true);
  const [isOpen, setIsOpen] = useState(true);
  const [filteringText, setFilteringText] = useState<string>('');
  const [resourceList, setResourceList] = useState<string[]>([]);

  useEffect(() => {
    getStackResourceList({
      stack_id: stackId,
      service_name: serviceName,
      resource_name: resourceName,
    })
      .then((list) => {
        setResourceList(list);
        console.log('resourceList', list);
      })
      .finally(() => setIsLoading(false));
  }, [stackId, serviceName, resourceName]);

  return (
    <LogicalIdTablePresentation
      isOpen={isOpen}
      isLoading={isLoading}
      resourceList={resourceList.filter((item) => item.toLowerCase().includes(filteringText.toLowerCase()))}
      filteringText={filteringText}
      onChangeFilteringText={({ detail }) => setFilteringText(detail.filteringText)}
      onCliCkLogicalId={(item) => (event) => {
        event.stopPropagation();
        modifyResourceTab(tabId, (originTab: ResourceTabInfo) => {
          return {
            ...originTab,
            selectedLogicalId: item,
          };
        });
      }}
      selectedLogicalId={selectedLogicalId}
      setToggleOpen={() => setIsOpen(!isOpen)}
    />
  );
};
