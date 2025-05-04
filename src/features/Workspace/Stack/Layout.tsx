import { useState } from 'react';
import {
  Outlet,
  useLoaderData,
  useNavigate,
  useOutletContext,
} from 'react-router';
import styled from 'styled-components';

import {
  AppLayout,
  BreadcrumbGroup,
  SideNavigation,
  SpaceBetween,
  TextFilter,
} from '@cloudscape-design/components';

import { useTemplates } from './hooks/useTemplates';
import { filterSideMenu } from './lib/FilterSideMenu';

import type { SideNavigationProps } from '@cloudscape-design/components';

import type { WorkspaceLayoutContext } from '../Layout';
import type { StackLayoutLoaderData } from './Loader';
export type StackLayoutContext = WorkspaceLayoutContext & StackLayoutLoaderData;

const StyledLink = styled.a`
  color: #424650; /* ホバーしていない時の色 */
  text-decoration: none;

  &:hover {
    color: #006ce0; /* ホバー時の色 */
  }
`;

export const StackLayout = (): JSX.Element => {
  const workspace = useOutletContext<WorkspaceLayoutContext>();
  const stackLoader = useLoaderData<StackLayoutLoaderData>();
  const navigate = useNavigate();
  const { sideMenu } = useTemplates();
  const [searchValue, setSearchValue] = useState<string>('');
  console.log('sideMenu', sideMenu);

  const outletContext: StackLayoutContext = {
    ...workspace,
    ...stackLoader,
  };

  const items: SideNavigationProps.Item[] = filterSideMenu(
    sideMenu,
    searchValue,
  );

  return (
    <AppLayout
      toolsHide
      breadcrumbs={
        <BreadcrumbGroup
          items={[
            { text: workspace.name, href: '#' },
            { text: stackLoader.stack.name, href: '#' },
          ]}
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
          items={items}
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
      content={<Outlet context={outletContext} />}
    />
  );
};
