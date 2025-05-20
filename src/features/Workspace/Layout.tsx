import { useCallback, useEffect, useState } from 'react';
import { Outlet, useLoaderData, useNavigate } from 'react-router';
import styled from 'styled-components';

import {
  AppLayout,
  Input,
  SideNavigation,
  SpaceBetween,
  TokenGroup,
} from '@cloudscape-design/components';

import { loadTemplateSummary } from '../../invoke/Stack';
import { filterSideMenu } from './lib/FilterSideMenu';

import type { TokenGroupProps } from '@cloudscape-design/components';
import type { TemplateSummary } from '../../invoke/Stack';

import type { Dispatch, SetStateAction } from 'react';
import type { WorkspaceLayoutLoaderData } from './Loader';

export type ResourceInfo =
  | {
      type: 'detail';
      tabId: string;
      stackId: string;
      stackName: string;
    }
  | {
      type: 'resource';
      tabId: string;
      stackId: string;
      stackName: string;
      serviceName: string;
      resourceName: string;
    };

export type WorkspaceLayoutContext = WorkspaceLayoutLoaderData & {
  resourceTabs: ResourceInfo[];
  setResourceTabs: Dispatch<SetStateAction<ResourceInfo[]>>;
  activeTabId: string | undefined;
  setActiveTabId: Dispatch<SetStateAction<string | undefined>>;
  loadTemplateSummaryWrap: () => Promise<void>;
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
  const [resourceTabs, setResourceTabs] = useState<ResourceInfo[]>([]);
  const [activeTabId, setActiveTabId] = useState<string | undefined>(undefined);
  const [sideMenu, setSideMenu] = useState<TemplateSummary[]>([]);

  const loadTemplateSummaryWrap = useCallback(() => {
    return loadTemplateSummary().then((summary) => {
      setSideMenu(summary);
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
            searchValue,
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
            const { href, text } = event.detail;
            if (href === '#') {
              // ワークスペース名をクリックしたとき
              setResourceTabs([]);
              navigate(`/workspaces/${workspace.id}`);
              return;
            }
            const [stackId, stackName, serviceName, resourceType] =
              href.split('/');
            const newTab: ResourceInfo = {
              type: text === 'Detail' ? 'detail' : 'resource',
              tabId: href,
              stackId: stackId,
              stackName: stackName,
              serviceName: serviceName,
              resourceName: resourceType,
            };
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
          <Outlet context={context} />
        </div>
      }
    />
  );
};
