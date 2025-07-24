import { useEffect, useState } from 'react';
import { Controller, useForm } from 'react-hook-form';
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
  Table,
} from '@cloudscape-design/components';
import { zodResolver } from '@hookform/resolvers/zod';

import { getStackOutputs, getStackParameters, loadStack, updateStackDetail } from '../../../../../invoke/Stack';
import { useWorkspaceResourceContext } from '../../../contexts';

import type { FlashbarProps } from '@cloudscape-design/components';
import type { OverviewTabAttr, OverviewTabInfo } from '../../../contexts';

export type OverviewTabProps = OverviewTabAttr;

const stackEditValidator = z.object({
  name: z.string().min(1).max(256),
  description: z.string().max(256),
});

type stackParametersDisplayProps = {
  name: string;
  type: string;
  description: string;
};

type stackOutputsDisplayProps = {
  name: string;
  description: string | null;
  exportName: string | null;
  value: string;
};

export type StackEditType = z.infer<typeof stackEditValidator>;

export type BuildOverviewTabNameProps = {
  stackName: string;
};
export function buildOverviewTabName({ stackName }: BuildOverviewTabNameProps): string {
  return stackName;
}

export const OverviewTab = (props: OverviewTabProps): JSX.Element => {
  const { modifyResourceTab } = useWorkspaceResourceContext();
  const [flashbarItems, setFlashbarItems] = useState<FlashbarProps.MessageDefinition[]>([]);
  const [stackParameters, setStackParameters] = useState<stackParametersDisplayProps[]>([]);
  const [stackOutputs, setStackOutputs] = useState<stackOutputsDisplayProps[]>([]);

  // biome-ignore lint/correctness/useExhaustiveDependencies: <explanation>
  useEffect(() => {
    loadStack(props.stackId).then((stackDetail) => {
      const stackName = stackDetail.name;
      const description = stackDetail.description_from_meta;

      // 取得したスタック情報を更新
      modifyResourceTab(props.tabId, (originTab: OverviewTabInfo) => {
        return {
          ...originTab,
          stackName: stackName,
          description: description,
        };
      });
    });

    // スタックパラメータを取得し、表中に表示する
    getStackParameters({ stack_id: props.stackId }).then((stackParameters) => {
      const keys = Object.keys(stackParameters);
      const parameters: stackParametersDisplayProps[] = keys.map((key) => {
        return {
          name: key,
          type: stackParameters[key].Type,
          description: stackParameters[key].Description,
        };
      });
      setStackParameters(parameters);
    });

    // スタックのoutputを取得し、表中に表示する
    getStackOutputs({ stack_id: props.stackId }).then((stackOutputs) => {
      const outputs = stackOutputs.map((output) => {
        return {
          name: output.name,
          description: output.description,
          exportName: output.exportName,
          value: JSON.stringify(output.value), // FIXME: convertIntrinsicFunctionValue関数を使用した文字列を代入する
        };
      });
      setStackOutputs(outputs);
    });
  }, []);

  const setIsEdit = (isEdit: boolean) => {
    // 編集モードに入る場合、リソースタブの情報を更新
    modifyResourceTab(props.tabId, (originTab: OverviewTabInfo) => {
      return {
        ...originTab,
        isEdit: isEdit,
      };
    });
  };

  return props.isEdit ? (
    <StackEditContent
      resourceTab={props}
      setIsEdit={setIsEdit}
      flashbarItems={flashbarItems}
      setFlashbarItems={setFlashbarItems}
    />
  ) : (
    <StackContent
      flashbarItems={flashbarItems}
      resourceTab={props}
      setIsEdit={setIsEdit}
      stackParameters={stackParameters}
      stackOutputs={stackOutputs}
    />
  );
};

type StackContentProps = {
  resourceTab: OverviewTabProps;
  setIsEdit: (isEdit: boolean) => void;
  flashbarItems: FlashbarProps.MessageDefinition[];
  stackParameters: stackParametersDisplayProps[];
  stackOutputs: stackOutputsDisplayProps[];
};

const StackContent = (props: StackContentProps): JSX.Element => {
  const { resourceTab, setIsEdit, flashbarItems, stackParameters, stackOutputs } = props;
  return (
    <ContentLayout
      header={
        <SpaceBetween size="m">
          <Header
            variant="h1"
            description={resourceTab.description}
            actions={
              <Button variant="normal" onClick={() => setIsEdit(true)}>
                編集
              </Button>
            }
          >
            {resourceTab ? resourceTab.stackName : ''}
          </Header>
          <Flashbar items={flashbarItems} />
        </SpaceBetween>
      }
    >
      <SpaceBetween size="m">
        <Table
          resizableColumns
          columnDefinitions={[
            {
              id: 'parameterName',
              header: 'Parameter Name',
              cell: (e) => e.name,
              isRowHeader: true,
              width: 250,
              minWidth: 150,
            },
            {
              id: 'type',
              header: 'Type',
              cell: (e) => <div style={{ whiteSpace: 'pre-line' }}>{e.type}</div>,
              width: 150,
              minWidth: 100,
            },
            {
              id: 'description',
              header: 'Description',
              cell: (e) => <div style={{ whiteSpace: 'pre-line' }}>{e.description}</div>,
              width: 500,
            },
          ]}
          stickyHeader
          enableKeyboardNavigation
          items={stackParameters}
          loadingText="Loading resources"
          trackBy="name"
          empty={
            <Box margin={{ vertical: 'xs' }} textAlign="center" color="inherit">
              <SpaceBetween size="m">
                <b>No Parameters</b>
              </SpaceBetween>
            </Box>
          }
          header={<Header>Parameter</Header>}
        />
        <Table
          resizableColumns
          columnDefinitions={[
            {
              id: 'outputName',
              header: 'Output Name',
              cell: (e) => e.name,
              isRowHeader: true,
              width: 250,
              minWidth: 150,
            },
            {
              id: 'export',
              header: 'Export',
              cell: (e) => <div style={{ whiteSpace: 'pre-line' }}>{e.exportName}</div>,
              width: 250,
              minWidth: 150,
            },
            {
              id: 'value',
              header: 'Value',
              cell: (e) => <div style={{ whiteSpace: 'pre-line' }}>{e.value}</div>,
              width: 250,
              minWidth: 150,
            },
            {
              id: 'description',
              header: 'Description',
              cell: (e) => <div style={{ whiteSpace: 'pre-line' }}>{e.description}</div>,
            },
          ]}
          stickyHeader
          enableKeyboardNavigation
          items={stackOutputs}
          loadingText="Loading resources"
          trackBy="name"
          empty={
            <Box margin={{ vertical: 'xs' }} textAlign="center" color="inherit">
              <SpaceBetween size="m">
                <b>No Outputs</b>
              </SpaceBetween>
            </Box>
          }
          header={<Header>Outputs</Header>}
        />
      </SpaceBetween>
    </ContentLayout>
  );
};

export type StackEditContentProps = {
  resourceTab: OverviewTabProps;
  setIsEdit: (isEdit: boolean) => void;
  flashbarItems: FlashbarProps.MessageDefinition[];
  setFlashbarItems: React.Dispatch<React.SetStateAction<FlashbarProps.MessageDefinition[]>>;
};

const StackEditContent = (props: StackEditContentProps): JSX.Element => {
  const { flashbarItems, resourceTab, setFlashbarItems, setIsEdit } = props;

  const { control, handleSubmit } = useForm<StackEditType>({
    mode: 'onChange',
    resolver: zodResolver(stackEditValidator),
    defaultValues: {
      name: resourceTab.stackName,
      description: resourceTab.description,
    },
  });
  const { modifyResourceTab } = useWorkspaceResourceContext();

  const onSave = async (stackDetailProps: StackEditType) => {
    try {
      await updateStackDetail({
        stack_id: resourceTab.stackId,
        name: stackDetailProps.name,
        description: stackDetailProps.description,
      });

      // 保存したスタックの情報を更新
      modifyResourceTab(resourceTab.tabId, (originTab: OverviewTabInfo) => {
        return {
          ...originTab,
          stackName: stackDetailProps.name,
          description: stackDetailProps.description,
        };
      });

      // Stack詳細表示画面に戻る
      setIsEdit(false);
    } catch (error) {
      const id = uuidV4();
      setFlashbarItems(() => [
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
      header={
        <SpaceBetween size="m">
          <Header>Stackの編集</Header>
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
