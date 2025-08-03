import { Controller } from 'react-hook-form';
import { z } from 'zod';

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

import type { Control, UseFormHandleSubmit } from 'react-hook-form';

import type { ButtonProps } from '@cloudscape-design/components';

export const StackEditValidator = z.object({
  name: z.string().min(1).max(256),
  description: z.string().max(256),
});
export type StackEditType = z.infer<typeof StackEditValidator>;

export type StackEditContentPresentationProps = {
  /**
   * Stackの編集フォームのコントロール
   */
  // biome-ignore lint/suspicious/noExplicitAny: <explanation>
  control: Control<StackEditType, any>;

  /**
   * 保存ボタンのクリックハンドラ
   */
  onSubmitSave: ReturnType<UseFormHandleSubmit<StackEditType, undefined>>;

  /**
   * キャンセルボタンのクリックハンドラ
   */
  onClickCancel: ButtonProps['onClick'];
};

export const StackEditContentPresentation = ({
  control,
  onSubmitSave,
  onClickCancel,
}: StackEditContentPresentationProps): JSX.Element => {
  return (
    <ContentLayout
      header={
        <SpaceBetween size="m">
          <Header>Stackの編集</Header>
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
                    label="Stack name"
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
                    label="Stack description"
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
