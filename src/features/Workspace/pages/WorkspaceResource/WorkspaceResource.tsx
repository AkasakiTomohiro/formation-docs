import { useEffect, useState } from 'react';
import { useLocation, useOutletContext } from 'react-router';

import { Tabs } from '@cloudscape-design/components';

import { ResourceTab, StackTab } from './components';

import type { ResourceInfo, WorkspaceLayoutContext } from '../../Layout';

export const WorkspaceResource = (): JSX.Element => {
  const location = useLocation();
  const { resourceTabs, setResourceTabs } =
    useOutletContext<WorkspaceLayoutContext>();

  const [activeTabId, setActiveTabId] = useState<string | undefined>(undefined);

  useEffect(() => {
    setResourceTabs(resourceTabs);
    setActiveTabId(resourceTabs[resourceTabs.length - 1]?.tabId);
  }, [resourceTabs, setResourceTabs]);

  // biome-ignore lint/correctness/useExhaustiveDependencies: <explanation>
  useEffect(() => {
    const stackId = location.state.selectedStackId;
    const stackName = location.state.selectedStackName;
    if (stackId && stackName) {
      const newTab: ResourceInfo = {
        type: 'detail',
        tabId: `${stackId}/${stackName}`,
        stackId,
        stackName,
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
  }, []);

  return (
    <>
      {activeTabId !== undefined && (
        <Tabs
          activeTabId={activeTabId}
          tabs={resourceTabs.map((tab) => {
            if (tab.type === 'detail') {
              return {
                id: tab.tabId,
                label: tab.stackName,
                content: (
                  <StackTab stackId={tab.stackId} stackName={tab.stackName} />
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
              label: `${tab.stackName} - ${tab.serviceName} - ${tab.resourceName}`,
              content: (
                <ResourceTab
                  stackId={tab.stackId}
                  stackName={tab.stackName}
                  serviceName={tab.serviceName}
                  resourceName={tab.resourceName}
                />
              ),
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
