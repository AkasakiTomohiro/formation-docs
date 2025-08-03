import { useRouteLoaderData } from 'react-router';

import { Link } from '@cloudscape-design/components';
import Box from '@cloudscape-design/components/box';
import Button from '@cloudscape-design/components/button';
import ContentLayout from '@cloudscape-design/components/content-layout';
import Flashbar from '@cloudscape-design/components/flashbar';
import Header from '@cloudscape-design/components/header';
import Pagination from '@cloudscape-design/components/pagination';
import SpaceBetween from '@cloudscape-design/components/space-between';
import Table from '@cloudscape-design/components/table';

import type { WorkspaceLayoutLoaderData } from '../../Loader';

import type { useCollection } from '@cloudscape-design/collection-hooks';
import type { ButtonProps, FlashbarProps, LinkProps, TableProps } from '@cloudscape-design/components';
import type { StackInfo } from '../../lib/StackInfo';

export type WorkspaceHomePresentationProps = {
  /**
   * ワークスペース編集ボタンクリック時のハンドラー
   */
  onClickWorkspaceEdit: ButtonProps['onClick'];

  /**
   * スタック名クリック時のハンドラー
   */
  onClickStackName: (item: StackInfo) => LinkProps['onClick'];

  /**
   * スタックインポートボタンクリック時のハンドラー
   */
  onClickImportStack: ButtonProps['onClick'];

  /**
   * スタック削除ボタンクリック時のハンドラー
   */
  onClickDeleteStack: ButtonProps['onClick'];

  /**
   * スタックの再読み込みボタンクリック時のハンドラー
   */
  onClickReloadStack: ButtonProps['onClick'];

  /**
   * 選択中のアイテム
   */
  selectedItems: StackInfo[];

  /**
   * 選択状態が変更されたときのハンドラー
   */
  onSelectionChange: TableProps['onSelectionChange'];

  /**
   * スタック一覧を読み込み中か
   */
  isLoading: boolean;

  /**
   * テーブルのコレクション
   */
  tableCollection: ReturnType<typeof useCollection<StackInfo>>;

  /**
   * フラッシュバーのアイテム
   */
  flashbarItems: FlashbarProps.MessageDefinition[];
};

export const WorkspaceHomePresentation = ({
  onClickWorkspaceEdit,
  onClickStackName,
  onClickImportStack,
  onClickDeleteStack,
  onClickReloadStack,
  selectedItems,
  onSelectionChange,
  isLoading,
  tableCollection,
  flashbarItems,
}: WorkspaceHomePresentationProps): JSX.Element => {
  const workspace = useRouteLoaderData('workspace') as WorkspaceLayoutLoaderData;

  return (
    <ContentLayout
      defaultPadding
      header={
        <SpaceBetween size="m">
          <Header
            variant="h1"
            description={workspace.description}
            actions={
              <SpaceBetween size="s">
                <Button variant="normal" onClick={onClickWorkspaceEdit}>
                  編集
                </Button>
              </SpaceBetween>
            }
          >
            {workspace.name}
          </Header>
          <Flashbar items={flashbarItems} />
        </SpaceBetween>
      }
    >
      <Table
        {...tableCollection.collectionProps}
        columnDefinitions={[
          {
            id: 'Name',
            header: 'Stack name',
            cell: (e) => <Link onClick={onClickStackName(e)}>{e.name}</Link>,
            isRowHeader: true,
          },
          {
            id: 'DescriptionForMeta',
            header: 'Description for Meta',
            cell: (e) => e.description_from_meta,
          },
          {
            id: 'DescriptionForStack',
            header: 'Description for Stack',
            cell: (e) => e.description_from_stack,
          },
        ]}
        selectionType="single"
        selectedItems={selectedItems}
        onSelectionChange={onSelectionChange}
        items={tableCollection.items}
        loadingText="Loading workspace"
        trackBy="name"
        empty={
          <Box margin={{ vertical: 'xs' }} textAlign="center" color="inherit">
            <SpaceBetween size="m">
              <b>No resources</b>
            </SpaceBetween>
          </Box>
        }
        header={
          <Header
            actions={
              <SpaceBetween direction="horizontal" size="xs">
                <Button onClick={onClickDeleteStack} disabled={selectedItems.length === 0}>
                  削除
                </Button>
                <Button onClick={onClickReloadStack}>更新</Button>
                <Button variant="primary" onClick={onClickImportStack}>
                  インポートスタック
                </Button>
              </SpaceBetween>
            }
          >
            スタック一覧
          </Header>
        }
        pagination={<Pagination {...tableCollection.paginationProps} />}
        loading={isLoading}
      />
    </ContentLayout>
  );
};
