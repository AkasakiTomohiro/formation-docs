import { AppLayout, SpaceBetween } from '@cloudscape-design/components';
import { useEffect } from 'react';
import { Outlet } from 'react-router';
import { WorkspaceSideMenu } from './components/WorkspaceSideMenu';
import { useWorkspaceResourceContext } from './contexts/WorkspaceResourceContext';

export const WorkspaceLayout = (): JSX.Element => {
  const { loadSideMenu, loadAllStackOutputs } = useWorkspaceResourceContext();

  // biome-ignore lint/correctness/useExhaustiveDependencies: false positive
  useEffect(() => {
    loadSideMenu();
    loadAllStackOutputs();
  }, []);

  return (
    <AppLayout
      toolsHide
      disableContentPaddings
      navigationOpen
      navigation={<WorkspaceSideMenu />}
      content={
        <div key="sample" style={{ margin: '16px' }}>
          <Outlet />
        </div>
      }
    />
  );
};
