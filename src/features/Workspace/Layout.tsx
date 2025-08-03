import { useEffect } from 'react';
import { Outlet, useLoaderData } from 'react-router';

import { AppLayout, Flashbar, SpaceBetween } from '@cloudscape-design/components';

import { useFlashbarContext } from '../../contexts/FlashbarContext';
import { WorkspaceSideMenu } from './components/WorkspaceSideMenu';
import { useWorkspaceResourceContext } from './contexts/WorkspaceResourceContext';

import type { WorkspaceLayoutLoaderData } from './Loader';

export type WorkspaceLayoutContext = WorkspaceLayoutLoaderData;

export const WorkspaceLayout = (): JSX.Element => {
  const context = useLoaderData<WorkspaceLayoutLoaderData>();
  const { loadSideMenu } = useWorkspaceResourceContext();
  const { flashbarItems } = useFlashbarContext();

  // biome-ignore lint/correctness/useExhaustiveDependencies: <explanation>
  useEffect(() => {
    loadSideMenu();
  }, []);

  return (
    <AppLayout
      toolsHide
      disableContentPaddings
      navigationOpen
      navigation={<WorkspaceSideMenu />}
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
