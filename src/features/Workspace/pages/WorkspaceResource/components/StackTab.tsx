import {
  Box,
  Button,
  Container,
  ContentLayout,
  Flashbar,
  FlashbarProps,
  FormField,
  Header,
  Input,
  SpaceBetween,
} from '@cloudscape-design/components';
import { zodResolver } from '@hookform/resolvers/zod';
import { useState } from 'react';
import { Controller, useForm } from 'react-hook-form';
import { z } from 'zod';

export type StackTabProps = {
  stackId: string;
  stackName: string;
};

const stackEditValidator = z.object({
  name: z.string().min(1).max(256),
  description: z.string().max(256),
});

export type StackEditType = z.infer<typeof stackEditValidator>;

export const StackTab = (props: StackTabProps): JSX.Element => {
  const [isEdit, setIsEdit] = useState(false);

  const StackContent = (): JSX.Element => (
    <ContentLayout
      header={
        <SpaceBetween size="m">
          <Header
            variant="h1"
            actions={
              <Button variant="normal" onClick={() => setIsEdit(true)}>
                編集
              </Button>
            }
          >
            {props.stackName}
          </Header>
          {/* <Flashbar items={flashbarItems} /> */}
        </SpaceBetween>
      }
    >
      <Container
        header={
          <Header variant="h2" description="Container description">
            Container header
          </Header>
        }
      >
        <div className="contentPlaceholder" />
        <p>{props.stackId}</p>
      </Container>
    </ContentLayout>
  );

  const StackEditContent = (): JSX.Element => {
    const { control, handleSubmit } = useForm<StackEditType>({
      mode: 'onChange',
      resolver: zodResolver(stackEditValidator),
      defaultValues: {
        name: props.stackName,
        // TODO: stackの説明を初期状態で表示する方法を考える
        description: '',
      },
    });

    // TODO: RustでStackの情報を更新する関数を実装し呼び出す
    const onSave = async () => {};

    return (
      <ContentLayout
        defaultPadding
        header={
          <SpaceBetween size="m">
            <Header>Stackの編集</Header>
            {/* TODO: フラッシュバーを追加 */}
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
                      label="Stack name"
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
                      label="Stack description"
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
                <Button variant="normal" onClick={() => setIsEdit(false)}>
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

  return isEdit ? <StackEditContent /> : <StackContent />;
};
