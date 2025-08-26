import { useEffect, useState } from 'react';

import { useWorkspaceResourceContext } from '../../../../../../contexts';
import { ResourceIdTablePresentation } from './ResourceIdTable.presentation';
import { getManualManagementResourceList } from './lib/GetManualManagementResourceList';

import type { ResourceIdTablePresentationProps } from './ResourceIdTable.presentation';

import type { ManualResourceTabInfo } from '../../../../../../contexts';
import type { ManualManagementResource } from './lib/GetManualManagementResourceList';

export type ResourceIdTableProps = {
  /**
   * タブID
   */
  tabId: string;

  /**
   * サービス名
   */
  serviceName: string;

  /**
   * リソース名
   */
  resourceName: string;

  /**
   * 選択されているリソースID
   */
  selectedResourceId: string | undefined;
};

export const ResourceIdTable = ({ tabId, serviceName, resourceName, selectedResourceId }: ResourceIdTableProps) => {
  const [isLoading, setIsLoading] = useState(true);
  const [isOpen, setIsOpen] = useState(true);
  const [resourceList, setResourceList] = useState<ManualManagementResource[]>([]);
  const [filteringText, setFilteringText] = useState<string>('');
  const { modifyResourceTab } = useWorkspaceResourceContext();

  useEffect(() => {
    getManualManagementResourceList({
      service_name: serviceName,
      resource_name: resourceName,
    })
      .then((list) => {
        setResourceList(list);
      })
      .finally(() => setIsLoading(false));
  }, [serviceName, resourceName]);

  const onClickResourceId: ResourceIdTablePresentationProps['onClickResourceId'] = (item) => (event) => {
    event.stopPropagation();
    modifyResourceTab(tabId, (originTab: ManualResourceTabInfo) => {
      return {
        ...originTab,
        selectedResourceId: item.resourceId,
      };
    });
  };

  return (
    <ResourceIdTablePresentation
      tabId={tabId}
      isOpen={isOpen}
      isLoading={isLoading}
      selectedResourceId={selectedResourceId}
      onClickExpand={() => {
        modifyResourceTab(tabId, (originTab: ManualResourceTabInfo) => {
          return {
            ...originTab,
            selectedResourceId: undefined,
          };
        });
      }}
      resourceList={resourceList.filter((item) => item.resourceId.toLowerCase().includes(filteringText.toLowerCase()))}
      filteringText={filteringText}
      onChangeFilteringText={({ detail }) => setFilteringText(detail.filteringText)}
      onClickResourceId={onClickResourceId}
      setToggleOpen={() => setIsOpen((prev) => !prev)}
    />
  );
};
