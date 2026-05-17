import { SideNavigation, SpaceBetween } from '@cloudscape-design/components';
import { useLoaderData } from 'react-router';
import styled from 'styled-components';
import { SearchForm } from './components';
import type { SideNavigationProps } from '@cloudscape-design/components';
import type { MouseEventHandler } from 'react';
import type { WorkspaceLayoutLoaderData } from '../../Loader';

export type WorkspaceSideMenuPresentationProps = {
  /**
   * サイドメニューアイテムがクリックされたときのハンドラー
   */
  onClickSideMenuItem: NonNullable<SideNavigationProps['onFollow']>;

  /**
   * サイドメニューのホームアイテムがクリックされたときのハンドラー
   */
  onClickSideMenuHome: MouseEventHandler<HTMLAnchorElement>;

  /**
   * サイドメニューアイテム
   */
  sidMenuItems: SideNavigationProps.Item[];

  /**
   * サイドメニューの「AWSリソースの説明」アイテムがクリックされたときのハンドラー
   */
  onClickSideMenuResourceDescriptionSettings: MouseEventHandler<HTMLAnchorElement>;
};

const StyledLink = styled.a`
  color: #424650; /* ホバーしていない時の色 */
  text-decoration: none;

  &:hover {
    color: #006ce0; /* ホバー時の色 */
  }
`;

export const WorkspaceSideMenuPresentation = (props: WorkspaceSideMenuPresentationProps): JSX.Element => {
  const workspace = useLoaderData<WorkspaceLayoutLoaderData>();

  return (
    <SideNavigation
      header={{
        href: '#',
        text: workspace.name,
      }}
      items={props.sidMenuItems}
      itemsControl={
        <SpaceBetween direction="vertical" size="m">
          <StyledLink href="#" onClick={props.onClickSideMenuHome}>
            ホーム
          </StyledLink>
          <SearchForm />
        </SpaceBetween>
      }
      onFollow={props.onClickSideMenuItem}
    />
  );
};
