import { Box, Button, ContentLayout, Header, SpaceBetween, Table } from '@cloudscape-design/components';

import type { ButtonProps } from '@cloudscape-design/components';

export type StackParametersDisplayProps = {
  name: string;
  type: string;
  description: string;
};

export type StackOutputsDisplayProps = {
  name: string;
  description: string | null;
  exportName: string | null;
  value: string;
};

export type StackDisplayContentPresentationProps = {
  /**
   * スタック名
   */
  stackName: string;

  /**
   * スタックの説明
   */
  stackDescription: string;

  /**
   * スタックのパラメータ情報
   */
  stackParameters: StackParametersDisplayProps[];

  /**
   * スタックの出力情報
   */
  stackOutputs: StackOutputsDisplayProps[];

  /**
   * 編集ボタンのクリックハンドラ
   */
  onClickEdit: ButtonProps['onClick'];
};

export const StackDisplayContentPresentation = ({
  stackName,
  stackDescription,
  stackParameters,
  stackOutputs,
  onClickEdit,
}: StackDisplayContentPresentationProps) => {
  return (
    <ContentLayout
      header={
        <SpaceBetween size="m">
          <Header
            variant="h1"
            description={stackDescription}
            actions={
              <Button variant="normal" onClick={onClickEdit}>
                編集
              </Button>
            }
          >
            {stackName}
          </Header>
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
