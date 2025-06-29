import { useEffect } from 'react';
import { useLocation, useOutletContext } from 'react-router';

import { Tabs } from '@cloudscape-design/components';

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

import type { ManualResourceTabProps } from './components/ManualResourceTab';

import type { WorkspaceLayoutContext } from '../../Layout';
import type { OverviewTabProps, ResourceTabProps } from './components';
type ResourceInfo<
  TType extends string,
  T extends { tabId: string } = { tabId: string },
> = {
  type: TType;
} & T;
export type WorkspaceResourceInfo =
  | ResourceInfo<'overview', OverviewTabProps>
  | ResourceInfo<'resource', ResourceTabProps>
  | ResourceInfo<'manualOverview'>
  | ResourceInfo<'manualResource', ManualResourceTabProps>;

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
        const newTab: WorkspaceResourceInfo = {
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
