import { useState } from 'react';
import { Controller, useForm } from 'react-hook-form';
import { useNavigate, useOutletContext, useRevalidator } from 'react-router';
import { v4 as uuidV4 } from 'uuid';
import { z } from 'zod';

import {
  Box,
  Button,
  Container,
  ContentLayout,
  Flashbar,
  FormField,
  Header,
  Input,
  SpaceBetween,
} from '@cloudscape-design/components';
import { zodResolver } from '@hookform/resolvers/zod';

import { updateWorkspace } from '../../../../invoke/Workspace';

import type { WorkspaceLayoutContext } from '../../Layout';

import type { FlashbarProps } from '@cloudscape-design/components';
const workspaceEditValidator = z.object({
  name: z.string().min(1).max(256),
  description: z.string().max(256),
});

type WorkspaceEditType = z.infer<typeof workspaceEditValidator>;

export const WorkspaceEdit = (): JSX.Element => {
  const workspace = useOutletContext<WorkspaceLayoutContext>();
  const navigate = useNavigate();
  const revalidator = useRevalidator();
  const [flashbarItems, setFlashbarItems] = useState<
    FlashbarProps.MessageDefinition[]
  >([]);
  const { control, handleSubmit } = useForm<WorkspaceEditType>({
    mode: 'onChange',
    resolver: zodResolver(workspaceEditValidator),
    defaultValues: {
      name: workspace.name,
      description: workspace.description,
    },
  });

  const onSave = async (newWorkspace: WorkspaceEditType) => {
    try {
      await updateWorkspace({
        name: newWorkspace.name,
        description: newWorkspace.description,
      });

      // ローダーの再読み込み
      await revalidator.revalidate();

      navigate(`/workspaces/${workspace.id}`);
    } catch (error) {
      const id = uuidV4();
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
            setFlashbarItems((items) => items.filter((e) => e.id !== id));
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
      <form onSubmit={handleSubmit(onSave)}>
        <SpaceBetween direction="vertical" size="m">
          <Container>
            <SpaceBetween direction="vertical" size="m">
              <Controller
                name="name"
                control={control}
                render={({ field, fieldState: { invalid } }) => (
                  <FormField
                    label="Workspace name"
                    errorText={
                      invalid
                        ? '1文字以上256文字以下で入力してください'
                        : undefined
                    }
                  >
                    <Input
                      {...field}
                      onChange={(event) => field.onChange(event.detail.value)}
                      invalid={invalid}
                    />
                  </FormField>
                )}
              />
              <Controller
                name="description"
                control={control}
                render={({ field, fieldState: { invalid } }) => (
                  <FormField
                    label="Workspace description"
                    errorText={
                      invalid ? '256文字以下で入力してください' : undefined
                    }
                  >
                    <Input
                      {...field}
                      onChange={(event) => field.onChange(event.detail.value)}
                      invalid={invalid}
                    />
                  </FormField>
                )}
              />
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
              <Button variant="primary" formAction="submit">
                保存
              </Button>
            </SpaceBetween>
          </Box>
        </SpaceBetween>
      </form>
    </ContentLayout>
  );
};
