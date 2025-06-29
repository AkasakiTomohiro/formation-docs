import { useCallback, useEffect, useState } from 'react';
import { Outlet, useLoaderData, useNavigate } from 'react-router';
import styled from 'styled-components';

import {
  AppLayout,
  Flashbar,
  Input,
  SideNavigation,
  SpaceBetween,
  TokenGroup,
} from '@cloudscape-design/components';

import { loadManualManagementResourceSummary } from '../../invoke/ManualManagementResource';
import { loadTemplateSummary } from '../../invoke/Stack';
import {
  ManualManagementId,
  filterSideMenu,
  hrefParser,
} from './lib/FilterSideMenu';

import type { WorkspaceResourceInfo } from './pages';

import type {
  FlashbarProps,
  TokenGroupProps,
} from '@cloudscape-design/components';
import type { TemplateSummary } from '../../invoke/Stack';

import type { Dispatch, SetStateAction } from 'react';
import type { WorkspaceLayoutLoaderData } from './Loader';

export type WorkspaceLayoutContext = WorkspaceLayoutLoaderData & {
  resourceTabs: WorkspaceResourceInfo[];
  setResourceTabs: Dispatch<SetStateAction<WorkspaceResourceInfo[]>>;
  activeTabId: string | undefined;
  setActiveTabId: Dispatch<SetStateAction<string | undefined>>;
  loadTemplateSummaryWrap: () => Promise<void>;
  flashbarItems: FlashbarProps.MessageDefinition[];
  setFlashbarItems: Dispatch<SetStateAction<FlashbarProps.MessageDefinition[]>>;
};

const StyledLink = styled.a`
  color: #424650; /* ホバーしていない時の色 */
  text-decoration: none;

  &:hover {
    color: #006ce0; /* ホバー時の色 */
  }
`;

export const WorkspaceLayout = (): JSX.Element => {
  const workspace = useLoaderData<WorkspaceLayoutLoaderData>();
  const navigate = useNavigate();
  const [searchValue, setSearchValue] = useState<string>('');
  const [tokenGroup, setTokenGroup] = useState<TokenGroupProps.Item[]>([]);
  const [resourceTabs, setResourceTabs] = useState<WorkspaceResourceInfo[]>([]);
  const [activeTabId, setActiveTabId] = useState<string | undefined>(undefined);
  const [sideMenu, setSideMenu] = useState<TemplateSummary[]>([]);
  const [flashbarItems, setFlashbarItems] = useState<
    FlashbarProps.MessageDefinition[]
  >([]);

  const loadTemplateSummaryWrap = useCallback(() => {
    return Promise.all([
      loadTemplateSummary(),
      loadManualManagementResourceSummary(),
    ]).then(([templateSummary, manualManagementSummary]) => {
      setSideMenu([
        ...templateSummary,
        {
          id: ManualManagementId,
          sectionGroupName: '手動管理リソース',
          resources: manualManagementSummary,
        },
      ]);
    });
  }, []);

  // biome-ignore lint/correctness/useExhaustiveDependencies: <explanation>
  useEffect(() => {
    loadTemplateSummaryWrap();
  }, []);

  const context: WorkspaceLayoutContext = {
    ...workspace,
    resourceTabs,
    setResourceTabs,
    activeTabId,
    setActiveTabId,
    loadTemplateSummaryWrap,
    flashbarItems,
    setFlashbarItems,
  };

  return (
    <AppLayout
      toolsHide
      disableContentPaddings
      navigationOpen={true}
      navigation={
        <SideNavigation
          header={{
            href: '#',
            text: workspace.name,
          }}
          items={filterSideMenu(sideMenu, [
            ...tokenGroup.map((t) => (t.label ? t.label : '')),
          ])}
          itemsControl={
            <SpaceBetween direction="vertical" size="m">
              <StyledLink
                href="#"
                onClick={(event) => {
                  event.preventDefault();
                  setResourceTabs([]);
                  navigate(`/workspaces/${workspace.id}`);
                }}
              >
                Home
              </StyledLink>
              <div>
                <Input
                  type="search"
                  value={searchValue}
                  placeholder="Search Resource"
                  ariaLabel="Search Resource"
                  onChange={({ detail }) => setSearchValue(detail.value)}
                  onKeyDown={({ detail }) => {
                    if (detail.key === 'Enter') {
                      if (searchValue === '') {
                        return;
                      }
                      setTokenGroup((prev) => [
                        ...prev,
                        { label: searchValue },
                      ]);
                      setSearchValue('');
                    }
                  }}
                />
                <TokenGroup
                  onDismiss={({ detail: { itemIndex } }) => {
                    setTokenGroup([
                      ...tokenGroup.slice(0, itemIndex),
                      ...tokenGroup.slice(itemIndex + 1),
                    ]);
                  }}
                  items={tokenGroup}
                />
              </div>
            </SpaceBetween>
          }
          onFollow={(event) => {
            event.preventDefault();
            console.log('onFollow', event.detail);
            const { href } = event.detail;
            if (href === '#') {
              // ワークスペース名をクリックしたとき
              setResourceTabs([]);
              navigate(`/workspaces/${workspace.id}`);
              return;
            }
            const hrefParts = hrefParser(href);
            let newTab: WorkspaceResourceInfo;
            switch (hrefParts.type) {
              case 'overview': {
                newTab = {
                  type: 'overview',
                  tabId: href,
                  stackId: hrefParts.stackId,
                  stackName: hrefParts.sectionGroupName,
                  description: '',
                };
                break;
              }
              case 'resource': {
                newTab = {
                  type: 'resource',
                  tabId: href,
                  stackId: hrefParts.stackId,
                  stackName: hrefParts.sectionGroupName,
                  serviceName: hrefParts.serviceName,
                  resourceName: hrefParts.resourceType,
                };
                break;
              }
              case 'manualOverview': {
                newTab = {
                  type: 'manualOverview',
                  tabId: href,
                };
                break;
              }
              case 'manualResource': {
                newTab = {
                  type: 'resource', // FIXME:
                  tabId: href,
                  stackId: ManualManagementId,
                  stackName: '手動管理リソース',
                  serviceName: hrefParts.serviceName,
                  resourceName: hrefParts.resourceType,
                };
                break;
              }
            }

            setResourceTabs((prev) => {
              const existingTab = prev.find(
                (tab) => tab.tabId === newTab.tabId,
              );
              if (existingTab) {
                return prev;
              }
              return [...prev, newTab];
            });
            setActiveTabId(href);
            navigate(`/workspaces/${workspace.id}/resources`);
          }}
        />
      }
      // notifications={
      //   <Flashbar
      //     items={[
      //       {
      //         type: 'info',
      //         dismissible: true,
      //         content: 'This is an info flash message.',
      //         id: 'message_1',
      //       },
      //     ]}
      //   />
      // }
      content={
        <div key="sample" style={{ margin: '16px' }}>
          <SpaceBetween size="m">
            <Flashbar items={flashbarItems} />
            <Outlet context={context} />
          </SpaceBetween>
        </div>
      }
    />
  );
};
