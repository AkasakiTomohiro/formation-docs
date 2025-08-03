import { useEffect } from 'react';
import { Outlet } from 'react-router';

import { AppLayout, Flashbar, SpaceBetween } from '@cloudscape-design/components';

import { useFlashbarContext } from '../../contexts/FlashbarContext';
import { WorkspaceSideMenu } from './components/WorkspaceSideMenu';
import { useWorkspaceResourceContext } from './contexts/WorkspaceResourceContext';

export const WorkspaceLayout = (): JSX.Element => {
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
            <Outlet />
          </SpaceBetween>
        </div>
      }
    />
  );
};
