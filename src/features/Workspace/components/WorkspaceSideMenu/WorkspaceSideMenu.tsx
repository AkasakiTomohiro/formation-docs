import { useLoaderData, useNavigate } from 'react-router';
import styled from 'styled-components';

import { Input, SideNavigation, SpaceBetween, TokenGroup } from '@cloudscape-design/components';

import { useWorkspaceResourceContext } from '../../contexts';
import { filterSideMenu, hrefParser } from '../../lib/FilterSideMenu';

import type { WorkspaceLayoutLoaderData } from '../../Loader';
import type { WorkspaceTabInfo } from '../../contexts';

const StyledLink = styled.a`
  color: #424650; /* ホバーしていない時の色 */
  text-decoration: none;

  &:hover {
    color: #006ce0; /* ホバー時の色 */
  }
`;

export const WorkspaceSideMenu = (): JSX.Element => {
  const workspace = useLoaderData<WorkspaceLayoutLoaderData>();
  const navigate = useNavigate();
  const { searchValue, setSearchValue, tokenGroup, setTokenGroup, addResourceTab, deleteAllResourceTabs, sideMenu } =
    useWorkspaceResourceContext();

  return (
    <SideNavigation
      header={{
        href: '#',
        text: workspace.name,
      }}
      items={filterSideMenu(sideMenu, [...tokenGroup.map((t) => (t.label ? t.label : ''))])}
      itemsControl={
        <SpaceBetween direction="vertical" size="m">
          <StyledLink
            href="#"
            onClick={(event) => {
              event.preventDefault();
              deleteAllResourceTabs();
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
                  setTokenGroup((prev) => [...prev, { label: searchValue }]);
                  setSearchValue('');
                }
              }}
            />
            <TokenGroup
              onDismiss={({ detail: { itemIndex } }) => {
                setTokenGroup([...tokenGroup.slice(0, itemIndex), ...tokenGroup.slice(itemIndex + 1)]);
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
          deleteAllResourceTabs();
          return;
        }
        const hrefParts = hrefParser(href);
        let newTab: WorkspaceTabInfo;
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
              type: 'manualResource',
              tabId: href,
              serviceName: hrefParts.serviceName,
              resourceName: hrefParts.resourceType,
            };
            break;
          }
        }
        addResourceTab(newTab);
        navigate(`/workspaces/${workspace.id}/resources`);
      }}
    />
  );
};
