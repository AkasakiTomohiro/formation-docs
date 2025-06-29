import { useEffect } from 'react';
import { useLocation, useOutletContext } from 'react-router';

import { Tabs } from '@cloudscape-design/components';

import {
  OverviewTab,
  ResourceTab,
  buildOverviewTabName,
  buildResourceTabName,
} from './components';
import { ManualOverviewTab } from './components/ManualOverviewTab';

import type { ResourceInfo, WorkspaceLayoutContext } from '../../Layout';
export const WorkspaceResource = (): JSX.Element => {
  const location = useLocation();
  const { resourceTabs, setResourceTabs, activeTabId, setActiveTabId } =
    useOutletContext<WorkspaceLayoutContext>();

  // biome-ignore lint/correctness/useExhaustiveDependencies: <explanation>
  useEffect(() => {
    if (location.state) {
      const stackId = location.state.selectedStackId;
      const stackName = location.state.selectedStackName;
      if (stackId && stackName) {
        const newTab: ResourceInfo = {
          type: 'overview',
          tabId: `${stackId}/${stackName}`,
          stackId,
          stackName,
          description: '',
        };
        setResourceTabs((prev) => {
          const existingTab = prev.find((tab) => tab.tabId === newTab.tabId);
          if (existingTab) {
            return prev;
          }
          return [...prev, newTab];
        });
        setActiveTabId(`${stackId}/${stackName}`);
      }
    }
  }, []);

  return (
    <>
      {activeTabId !== undefined && (
        <Tabs
          activeTabId={activeTabId}
          tabs={resourceTabs.map((tab) => {
            if (tab.type === 'overview') {
              return {
                id: tab.tabId,
                label: buildOverviewTabName({ stackName: tab.stackName }),
                // FIXME:
                content:
                  tab.stackId === 'manualManagement' ? (
                    <ManualOverviewTab
                      stackId={tab.stackId}
                      sectionGroupName={tab.stackName}
                    />
                  ) : (
                    <OverviewTab {...tab} />
                  ),
                dismissible: true,
                onDismiss: () => {
                  setResourceTabs((prev) =>
                    prev.filter((t) => t.tabId !== tab.tabId),
                  );
                },
              };
            }
            return {
              id: tab.tabId,
              label: buildResourceTabName({
                stackName: tab.stackName,
                serviceName: tab.serviceName,
                resourceName: tab.resourceName,
              }),
              content: <ResourceTab {...tab} />,
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
