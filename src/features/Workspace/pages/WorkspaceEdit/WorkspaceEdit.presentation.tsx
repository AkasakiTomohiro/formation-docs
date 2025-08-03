import { Controller } from 'react-hook-form';

import {
  Box,
  Button,
  Container,
  ContentLayout,
  FormField,
  Header,
  Input,
  SpaceBetween,
} from '@cloudscape-design/components';

import type { WorkspaceEditType } from './WorkspaceEdit';

import type { Control, UseFormHandleSubmit } from 'react-hook-form';

import type { ButtonProps } from '@cloudscape-design/components';

export type WorkspaceEditPresentationProps = {
  /**
   * useFormのコントロール
   */
  // biome-ignore lint/suspicious/noExplicitAny: <explanation>
  control: Control<WorkspaceEditType, any>;

  /**
   * 保存ボタンのクリックイベントハンドラ
   */
  onSubmitSave: ReturnType<UseFormHandleSubmit<WorkspaceEditType, undefined>>;

  /**
   * キャンセルボタンのクリックイベントハンドラ
   */
  onClickCancel: ButtonProps['onClick'];
};

export const WorkspaceEditPresentation = ({
  control,
  onSubmitSave,
  onClickCancel,
}: WorkspaceEditPresentationProps): JSX.Element => {
  return (
    <ContentLayout
      defaultPadding
      header={
        <SpaceBetween size="m">
          <Header>Workspaceの編集</Header>
        </SpaceBetween>
      }
    >
      <form onSubmit={onSubmitSave}>
        <SpaceBetween direction="vertical" size="m">
          <Container>
            <SpaceBetween direction="vertical" size="m">
              <Controller
                name="name"
                control={control}
                render={({ field, fieldState: { invalid } }) => (
                  <FormField
                    label="Workspace name"
                    errorText={invalid ? '1文字以上256文字以下で入力してください' : undefined}
                  >
                    <Input {...field} onChange={(event) => field.onChange(event.detail.value)} invalid={invalid} />
                  </FormField>
                )}
              />
              <Controller
                name="description"
                control={control}
                render={({ field, fieldState: { invalid } }) => (
                  <FormField
                    label="Workspace description"
                    errorText={invalid ? '256文字以下で入力してください' : undefined}
                  >
                    <Input {...field} onChange={(event) => field.onChange(event.detail.value)} invalid={invalid} />
                  </FormField>
                )}
              />
            </SpaceBetween>
          </Container>
          <Box float="right">
            <SpaceBetween direction="horizontal" size="xs">
              <Button variant="normal" onClick={onClickCancel}>
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
