import {
  Box,
  Button,
  ContentLayout,
  Flashbar,
  Header,
  Link,
  Pagination,
  SpaceBetween,
  Table,
} from '@cloudscape-design/components';
import type { useCollection } from '@cloudscape-design/collection-hooks';
import type { ButtonProps, FlashbarProps, LinkProps, TableProps } from '@cloudscape-design/components';
import type { WorkspaceExpand } from '../../hooks/useWorkspaces';

export type AppHomePresentationProps = {
  /**
   * フラッシュバーのアイテム
   */
  flashbarItems: FlashbarProps.MessageDefinition[];

  /**
   * ワークスペースの読み込み中かどうか
   */
  isLoading: boolean;

  /**
   * 選択中のワークスペース
   */
  selectedItems: WorkspaceExpand[];

  /**
   * テーブルのコレクション
   */
  tableCollection: ReturnType<typeof useCollection<WorkspaceExpand>>;

  /**
   * ワークスペースのリンククリック時のハンドラー
   */
  onClickWorkspaceLink: (item: WorkspaceExpand) => LinkProps['onClick'];

  /**
   * ワークスペース新規作成ボタンクリック時のハンドラー
   */
  onClickNewWorkspace: ButtonProps['onClick'];

  /**
   * ワークスペース削除ボタンクリック時のハンドラー
   */
  onClickDeleteWorkspace: ButtonProps['onClick'];

  /**
   * ワークスペース更新ボタンクリック時のハンドラー
   */
  onClickUpdateWorkspace: ButtonProps['onClick'];

  /**
   * 選択状態が変更されたときのハンドラー
   */
  onSelectionChange: TableProps['onSelectionChange'];
};

export const AppHomePresentation = ({
  flashbarItems,
  isLoading,
  selectedItems,
  tableCollection,
  onClickWorkspaceLink,
  onClickNewWorkspace,
  onClickDeleteWorkspace,
  onClickUpdateWorkspace,
  onSelectionChange,
}: AppHomePresentationProps): JSX.Element => {
  return (
    <ContentLayout
      defaultPadding
      header={
        <SpaceBetween size="m">
          <Header variant="h1">Home</Header>
          <Flashbar items={flashbarItems} />
        </SpaceBetween>
      }
    >
      <Table
        {...tableCollection.collectionProps}
        columnDefinitions={[
          {
            id: 'Name',
            header: 'Workspace name',
            cell: (e) => <Link onClick={onClickWorkspaceLink(e)}>{e.name}</Link>,
            isRowHeader: true,
          },
          {
            id: 'Directory',
            header: 'directory',
            cell: (e) => e.directory,
          },
          {
            id: 'Description',
            header: 'Description',
            cell: (e) => e.description,
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
                <Button onClick={onClickDeleteWorkspace} disabled={selectedItems.length === 0}>
                  削除
                </Button>
                <Button onClick={onClickUpdateWorkspace}>更新</Button>
                <Button variant="primary" onClick={onClickNewWorkspace}>
                  新規ワークスペース
                </Button>
              </SpaceBetween>
            }
          >
            ワークスペース一覧
          </Header>
        }
        pagination={<Pagination {...tableCollection.paginationProps} />}
        loading={isLoading}
      />
    </ContentLayout>
  );
};
