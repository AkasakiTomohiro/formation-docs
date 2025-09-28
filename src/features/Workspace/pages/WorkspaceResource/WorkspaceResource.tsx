import { Tabs } from '@cloudscape-design/components';
import { useEffect } from 'react';
import { useLocation } from 'react-router';
import { hrefBuilder } from '../../components';
import { useWorkspaceResourceContext } from '../../contexts/WorkspaceResourceContext';
import {
  buildManualOverviewTabName,
  buildOverviewTabName,
  buildResourceTabName,
  ManualOverviewTab,
  OverviewTab,
  ResourceTab,
} from './components';
import { buildManualResourceTabName, ManualResourceTab } from './components/ManualResourceTab';
import type { WorkspaceTabInfo } from '../../contexts';
export const WorkspaceResource = (): JSX.Element => {
  const location = useLocation();
  const { resourceTabs, addResourceTab, deleteResourceTab, activeTabId, setActiveTabId } =
    useWorkspaceResourceContext();

  // biome-ignore lint/correctness/useExhaustiveDependencies: false positive
  useEffect(() => {
    if (location.state) {
      const stackId = location.state.selectedStackId;
      const stackName = location.state.selectedStackName;
      if (stackId && stackName) {
        const tabId = hrefBuilder({
          type: 'overview',
          stackId: stackId,
          sectionGroupName: stackName,
        });
        const newTab: WorkspaceTabInfo = {
          type: 'overview',
          tabId,
          stackId,
          stackName,
          description: '',
        };
        addResourceTab(newTab);
      }
    }
  }, []);

  return (
    <>
      {activeTabId !== undefined && (
        <Tabs
          activeTabId={activeTabId}
          tabs={resourceTabs.map((tab) => {
            let label: string;
            let content: JSX.Element;
            switch (tab.type) {
              case 'overview': {
                label = buildOverviewTabName({ stackName: tab.stackName });
                content = <OverviewTab {...tab} />;
                break;
              }
              case 'resource': {
                label = buildResourceTabName({
                  stackName: tab.stackName,
                  serviceName: tab.serviceName,
                  resourceName: tab.resourceName,
                });
                content = <ResourceTab {...tab} />;
                break;
              }
              case 'manualOverview': {
                label = buildManualOverviewTabName();
                content = <ManualOverviewTab />;
                break;
              }
              case 'manualResource': {
                label = buildManualResourceTabName({
                  serviceName: tab.serviceName,
                  resourceName: tab.resourceName,
                });
                content = <ManualResourceTab {...tab} />;
                break;
              }
            }
            return {
              id: tab.tabId,
              label: label,
              content: content,
              dismissible: true,
              onDismiss: () => {
                deleteResourceTab(tab.tabId);
              },
            };
          })}
          onChange={(event) => {
            setActiveTabId(event.detail.activeTabId);
          }}
        />
      )}
    </>
  );
};
