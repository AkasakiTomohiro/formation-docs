import { useCallback, useMemo } from 'react';
import { useLoaderData, useNavigate } from 'react-router';
import { useWorkspaceResourceContext } from '../../contexts';
import { filterSideMenu, hrefParser } from './lib';
import { WorkspaceSideMenuPresentation } from './WorkspaceSideMenu.presentation';
import type { WorkspaceTabInfo } from '../../contexts';
import type { WorkspaceLayoutLoaderData } from '../../Loader';
import type { WorkspaceSideMenuPresentationProps } from './WorkspaceSideMenu.presentation';

export const WorkspaceSideMenu = (): JSX.Element => {
  const workspace = useLoaderData<WorkspaceLayoutLoaderData>();
  const navigate = useNavigate();
  const { tokenGroup, addResourceTab, deleteAllResourceTabs, sideMenu } = useWorkspaceResourceContext();

  const onClickSideMenuItem = useCallback<WorkspaceSideMenuPresentationProps['onClickSideMenuItem']>(
    (event) => {
      event.preventDefault();
      const { href } = event.detail;

      if (href === '#') {
        // ワークスペース名をクリックしたとき
        deleteAllResourceTabs();
        return;
      }

      // リソース名・Overviewをクリックしたとき
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
            viewMode: 'value',
          };
          break;
        }
      }

      addResourceTab(newTab);
      navigate(`/workspaces/${workspace.id}/resources`);
    },
    [addResourceTab, deleteAllResourceTabs, navigate, workspace.id],
  );

  const sidMenuItems: WorkspaceSideMenuPresentationProps['sidMenuItems'] = useMemo(() => {
    return filterSideMenu(sideMenu, [...tokenGroup.map((t) => (t.label ? t.label : ''))]);
  }, [sideMenu, tokenGroup]);

  return (
    <WorkspaceSideMenuPresentation
      sidMenuItems={sidMenuItems}
      onClickSideMenuItem={onClickSideMenuItem}
      onClickSideMenuHome={(event) => {
        event.preventDefault();
        deleteAllResourceTabs();
      }}
    />
  );
};
