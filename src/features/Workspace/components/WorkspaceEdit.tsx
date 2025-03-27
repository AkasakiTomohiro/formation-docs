import {
  Box,
  Button,
  Container,
  ContentLayout,
  Flashbar,
  type FlashbarProps,
  FormField,
  Header,
  Input,
  SpaceBetween,
} from '@cloudscape-design/components';
import { useState } from 'react';
import { useNavigate, useOutletContext, useRevalidator } from 'react-router';
import { v4 as uuidv4 } from 'uuid';
import { updateWorkspace } from '../../../invoke/Workspace';
import type { WorkspaceLayoutLoaderData } from '../Loader';

export const WorkspaceEdit = (): JSX.Element => {
  const workspace = useOutletContext<WorkspaceLayoutLoaderData>();
  const navigate = useNavigate();
  const revalidator = useRevalidator();
  const [flashbarItems, setFlashbarItems] = useState<
    FlashbarProps.MessageDefinition[]
  >([]);
  const [workspaceName, setWorkspaceName] = useState(workspace.name);
  const [workspaceDescription, setWorkspaceDescription] = useState(
    workspace.description,
  );

  const handleSave = async () => {
    try {
      // FIXME: バリデーションチェックを行う
      await updateWorkspace(workspace.id, {
        name: workspaceName,
        description: workspaceDescription,
      });

      // ローダーの再読み込み
      await revalidator.revalidate();

      navigate(`/workspaces/${workspace.id}`);
    } catch (error) {
      const id = uuidv4();
      setFlashbarItems([
        ...flashbarItems,
        {
          type: 'error',
          header: '保存に失敗しました',
          content: typeof error === 'string' ? error : undefined,
          dismissible: true,
          dismissLabel: 'close',
          id: id,
          onDismiss: () => {
            setFlashbarItems(flashbarItems.filter((e) => e.id !== id));
          },
        },
      ]);
    }
  };

  return (
    <ContentLayout
      defaultPadding
      header={
        <SpaceBetween size="m">
          <Header>Workspaceの編集</Header>
          <Flashbar items={flashbarItems} />
        </SpaceBetween>
      }
    >
      <SpaceBetween direction="vertical" size="m">
        <Container>
          <SpaceBetween direction="vertical" size="m">
            <FormField label="Workspace name">
              <Input
                value={workspaceName}
                onChange={(event) => setWorkspaceName(event.detail.value)}
              />
            </FormField>
            <FormField label="Workspace description">
              <Input
                value={workspaceDescription}
                onChange={(event) =>
                  setWorkspaceDescription(event.detail.value)
                }
              />
            </FormField>
          </SpaceBetween>
        </Container>
        <Box float="right">
          <SpaceBetween direction="horizontal" size="xs">
            <Button
              variant="normal"
              onClick={() => navigate(`/workspaces/${workspace.id}`)}
            >
              キャンセル
            </Button>
            <Button variant="primary" onClick={handleSave}>
              保存
            </Button>
          </SpaceBetween>
        </Box>
      </SpaceBetween>
    </ContentLayout>
  );
};
