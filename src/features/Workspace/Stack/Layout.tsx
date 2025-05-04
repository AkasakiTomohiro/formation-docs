import { useState } from 'react';
import { useNavigate, useOutletContext } from 'react-router';
import styled from 'styled-components';

import {
  AppLayout,
  BreadcrumbGroup,
  SideNavigation,
  SpaceBetween,
  Tabs,
  TextFilter,
} from '@cloudscape-design/components';

import { useTemplates } from './hooks/useTemplates';
import { filterSideMenu } from './lib/FilterSideMenu';
import { StackHome } from './pages';

import type { WorkspaceLayoutContext } from '../Layout';

const StyledLink = styled.a`
  color: #424650; /* ホバーしていない時の色 */
  text-decoration: none;

  &:hover {
    color: #006ce0; /* ホバー時の色 */
  }
`;

export type ResourceInfo = {
  type: 'detail';
  tabId: string;
  stackId: string;
  stackName: string;
};

export const StackLayout = (): JSX.Element => {
  const workspace = useOutletContext<WorkspaceLayoutContext>();
  const navigate = useNavigate();
  const { sideMenu } = useTemplates();
  const [searchValue, setSearchValue] = useState<string>('');
  const [resourceTabs, setResourceTabs] = useState<ResourceInfo[]>([]);
  const [activeTabId, setActiveTabId] = useState<string | undefined>(undefined);

  return (
    <AppLayout
      toolsHide
      breadcrumbs={
        <BreadcrumbGroup
          items={[{ text: workspace.name, href: '#' }]}
          onClick={(e) => {
            if (e.detail.text === workspace.name) {
              navigate(`/workspaces/${workspace.id}`);
            }
          }}
        />
      }
      navigationOpen={true}
      navigation={
        <SideNavigation
          header={{
            href: '#',
            text: workspace.name,
          }}
          items={filterSideMenu(sideMenu, searchValue)}
          itemsControl={
            <SpaceBetween direction="vertical" size="m">
              <StyledLink href="#">Home</StyledLink>
              <TextFilter
                filteringText={searchValue}
                filteringPlaceholder="Search Resource"
                filteringAriaLabel="Search Resource"
                onChange={({ detail }) => setSearchValue(detail.filteringText)}
              />
            </SpaceBetween>
          }
          onFollow={(event) => {
            event.preventDefault();
            console.log('onFollow', event.detail);
            const { href, text } = event.detail;
            if (text === 'Detail') {
              const [stackId, stackName] = href.split('/');
              const newTab: ResourceInfo = {
                type: 'detail',
                tabId: href,
                stackId: stackId,
                stackName: stackName,
              };
              setActiveTabId(newTab.tabId);
              setResourceTabs((prev) => {
                const existingTab = prev.find(
                  (tab) => tab.tabId === newTab.tabId,
                );
                if (existingTab) {
                  return prev;
                }
                return [...prev, newTab];
              });
            }
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
        activeTabId !== undefined && (
          <Tabs
            activeTabId={activeTabId}
            tabs={resourceTabs.map((tab) => {
              return {
                id: tab.tabId,
                label: tab.stackName,
                content: (
                  <StackHome stackId={tab.stackId} stackName={tab.stackName} />
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
        )
      }
    />
  );
};
