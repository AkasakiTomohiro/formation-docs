import { useState } from 'react';
import { Outlet, useLoaderData, useNavigate } from 'react-router';
import styled from 'styled-components';

import {
  AppLayout,
  SideNavigation,
  SpaceBetween,
  TextFilter,
} from '@cloudscape-design/components';

import { useTemplates } from './hooks/useTemplates';
import { filterSideMenu } from './lib/FilterSideMenu';

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
  const { sideMenu } = useTemplates();
  const [searchValue, setSearchValue] = useState<string>('');
  const [resourceTabs, setResourceTabs] = useState<ResourceInfo[]>([]);

  const context: WorkspaceLayoutContext = {
    ...workspace,
    resourceTabs,
    setResourceTabs,
  };

  return (
    <AppLayout
      toolsHide
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
              <StyledLink
                href="#"
                onClick={(event) => {
                  event.preventDefault();
                  navigate(`/workspaces/${workspace.id}`);
                }}
              >
                Home
              </StyledLink>
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
      content={<Outlet context={context} />}
    />
  );
};
