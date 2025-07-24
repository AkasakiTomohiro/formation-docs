import { useEffect, useState } from 'react';
import { Outlet, useLoaderData } from 'react-router';

import { AppLayout, Flashbar, SpaceBetween } from '@cloudscape-design/components';

import { WorkspaceSideMenu } from './components/WorkspaceSideMenu';
import { useWorkspaceResourceContext } from './contexts/WorkspaceResourceContext';

import type { FlashbarProps } from '@cloudscape-design/components';

import type { Dispatch, SetStateAction } from 'react';
import type { WorkspaceLayoutLoaderData } from './Loader';

export type WorkspaceLayoutContext = WorkspaceLayoutLoaderData & {
  flashbarItems: FlashbarProps.MessageDefinition[];
  setFlashbarItems: Dispatch<SetStateAction<FlashbarProps.MessageDefinition[]>>;
};

export const WorkspaceLayout = (): JSX.Element => {
  const workspace = useLoaderData<WorkspaceLayoutLoaderData>();
  const { loadSideMenu } = useWorkspaceResourceContext();

  // エラー表示
  const [flashbarItems, setFlashbarItems] = useState<FlashbarProps.MessageDefinition[]>([]);

  // biome-ignore lint/correctness/useExhaustiveDependencies: <explanation>
  useEffect(() => {
    loadSideMenu();
  }, []);

  const context: WorkspaceLayoutContext = {
    ...workspace,
    flashbarItems,
    setFlashbarItems,
  };

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
