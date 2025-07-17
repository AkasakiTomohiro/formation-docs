import { useEffect } from 'react';
import { useLocation } from 'react-router';

import { Tabs } from '@cloudscape-design/components';

import { useWorkspaceResourceContext } from '../../contexts/WorkspaceResourceContext';
import {
  OverviewTab,
  ResourceTab,
  buildManualOverviewTabName,
  buildOverviewTabName,
  buildResourceTabName,
} from './components';
import { ManualOverviewTab } from './components/ManualOverviewTab';
import {
  ManualResourceTab,
  buildManualResourceTabName,
} from './components/ManualResourceTab';

import type { ManualResourceTabInfo } from './components/ManualResourceTab';

import type {
  ManualOverviewTabInfo,
  OverviewTabInfo,
  ResourceTabInfo,
} from './components';

export type WorkspaceResourceInfo =
  | OverviewTabInfo
  | ResourceTabInfo
  | ManualOverviewTabInfo
  | ManualResourceTabInfo;

export const WorkspaceResource = (): JSX.Element => {
  const location = useLocation();
  const {
    resourceTabs,
    setResourceTabs,
    addResourceTab,
    activeTabId,
    setActiveTabId,
  } = useWorkspaceResourceContext();

  // biome-ignore lint/correctness/useExhaustiveDependencies: <explanation>
  useEffect(() => {
    if (location.state) {
      const stackId = location.state.selectedStackId;
      const stackName = location.state.selectedStackName;
      if (stackId && stackName) {
        const newTab: WorkspaceResourceInfo = {
          type: 'overview',
          tabId: `${stackId}/${stackName}`,
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
                setResourceTabs((prev) =>
                  prev.filter((t) => t.tabId !== tab.tabId),
                );
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
