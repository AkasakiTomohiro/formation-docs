import { useCollection } from '@cloudscape-design/collection-hooks';
import { Window } from '@tauri-apps/api/window';
import { open } from '@tauri-apps/plugin-dialog';
import { useCallback, useEffect, useRef, useState } from 'react';
import { useFlashbarContext } from '../../contexts/FlashbarContext';
import { useWorkspaces } from '../../hooks/useWorkspaces';
import { openWorkspace } from './lib/OpenWorkspace';
import { WorkspacesPresentation } from './Workspaces.presentation';
import type { WorkspaceExpand } from '../../hooks/useWorkspaces';
import type { WorkspacesPresentationProps } from './Workspaces.presentation';

export const Workspaces = (): JSX.Element => {
  const isFirstRender = useRef(true);
  const { flashbarItems, addFlashbarItem } = useFlashbarContext();
  const { state, workspaces, createWorkspace, loadWorkspaces, deleteWorkspace } = useWorkspaces();
  const tableCollection = useCollection(workspaces, {
    pagination: { pageSize: 10 },
  });
  const [selectedItems, setSelectedItems] = useState<WorkspaceExpand[]>([]);

  const openWorkspaceWrap = useCallback<WorkspacesPresentationProps['onClickWorkspaceLink']>(
    (workspace: WorkspaceExpand) => async (_) => {
      const workspaceWindow = await Window.getByLabel(`workspace-${workspace.id}`);
      if (workspaceWindow) {
        // ワークスペースが開いている場合は、フォーカスを当てる
        workspaceWindow.setFocus();
      } else {
        // ワークスペースが開いていない場合は、新しいウィンドウで開く
        const result = await openWorkspace(workspace.id);
        console.log({ result, workspace });
        if (!result) {
          addFlashbarItem({
            type: 'error',
            header: 'Workspaceが開けませんでした',
            content: `「${workspace.directory}」が存在することを確認してください`,
          });
        }
      }
    },
    [addFlashbarItem],
  );

  const newWorkspace = useCallback(async () => {
    const selectedDir = await open({
      multiple: false,
      directory: true,
      defaultPath: '~/Desktop',
    });
    if (selectedDir != null) {
      createWorkspace({ directory: selectedDir })
        .then((workspace) => openWorkspaceWrap(workspace))
        .catch((error) => {
          addFlashbarItem({
            type: 'error',
            header: '新規Workspaceの読み込みに失敗しました',
            content: typeof error === 'string' ? error : undefined,
          });
        });
    }
  }, [createWorkspace, openWorkspaceWrap, addFlashbarItem]);

  const onClickDeleteWorkspace = () => {
    deleteWorkspace(selectedItems[0]);
    setSelectedItems([]);
  };

  // biome-ignore lint/correctness/useExhaustiveDependencies: false positive
  useEffect(() => {
    if (isFirstRender.current) {
      isFirstRender.current = false;
      return;
    }
    loadWorkspaces().catch((error) => {
      addFlashbarItem({
        type: 'error',
        header: 'Workspaceの読み込みに失敗しました',
        content: typeof error === 'string' ? error : undefined,
      });
    });
  }, []);

  return (
    <WorkspacesPresentation
      flashbarItems={flashbarItems}
      isLoading={state === 'loading'}
      selectedItems={selectedItems}
      tableCollection={tableCollection}
      onClickWorkspaceLink={openWorkspaceWrap}
      onClickNewWorkspace={newWorkspace}
      onClickDeleteWorkspace={onClickDeleteWorkspace}
      onClickUpdateWorkspace={loadWorkspaces}
      onSelectionChange={({ detail }) => setSelectedItems(detail.selectedItems)}
    />
  );
};
