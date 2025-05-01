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
  Link,
  SideNavigation,
  SpaceBetween,
  TextFilter,
} from '@cloudscape-design/components';

import { useTemplates } from './hooks/UseTemplates';

import type { SideNavigationProps } from '@cloudscape-design/components';

import type { WorkspaceLayoutContext } from '../Layout';
import type { StackLayoutLoaderData } from './Loader';
export type StackLayoutContext = WorkspaceLayoutContext & StackLayoutLoaderData;

type SectionGroupItem =
  | SideNavigationProps.Section
  | SideNavigationProps.Link
  | SideNavigationProps.LinkGroup
  | SideNavigationProps.ExpandableLinkGroup;

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

  const items: SideNavigationProps.Item[] = [];
  for (const item of sideMenu.sort((a, b) =>
    a.stackName.localeCompare(b.stackName),
  )) {
    const newChildren: SectionGroupItem[] = [];
    if (item.stackName.toLowerCase().includes(searchValue.toLowerCase())) {
      // スタック名に検索文字が含まれている場合はすべてのリソースを表示する
      for (const resource of item.resources.sort((a, b) =>
        a.serviceName.localeCompare(b.serviceName),
      )) {
        newChildren.push({
          type: 'section',
          text: resource.serviceName,
          items: resource.recourseType
            .sort((a, b) => a.localeCompare(b))
            .map((rType) => ({
              type: 'link',
              text: rType,
              href: '#',
            })),
        });
      }
    } else {
      // スタック名に検索文字が含まれていない場合は、サービス名もしくはリソース名に検索文字が含まれているものを表示する
      for (const resource of item.resources.sort((a, b) =>
        a.serviceName.localeCompare(b.serviceName),
      )) {
        if (
          resource.serviceName.toLowerCase().includes(searchValue.toLowerCase())
        ) {
          // サービス名に検索文字が含まれている場合は、すべてのリソースを表示する
          newChildren.push({
            type: 'section',
            text: resource.serviceName,
            items: resource.recourseType
              .sort((a, b) => a.localeCompare(b))
              .map((rType) => ({
                type: 'link',
                text: rType,
                href: '#',
              })),
          });
        } else {
          // サービス名に検索文字が含まれていない場合は、リソース名に検索文字が含まれているものを表示する
          const newResource: SideNavigationProps.Section = {
            type: 'section',
            text: resource.serviceName,
            items: resource.recourseType
              .sort((a, b) => a.localeCompare(b))
              .filter((rType) =>
                rType.toLowerCase().includes(searchValue.toLowerCase()),
              )
              .map((rType) => ({
                type: 'link',
                text: rType,
                href: '#',
              })),
          };
          if (newResource.items.length > 0) {
            newChildren.push(newResource);
          }
        }
      }
    }
    if (newChildren.length === 0) {
      continue;
    }
    const newItem: SideNavigationProps.SectionGroup = {
      type: 'section-group',
      title: item.stackName,
      items: newChildren,
    };
    items.push(newItem);
    items.push({ type: 'divider' });
  }

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
              <Link href="#" variant="primary">
                Home
              </Link>
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
