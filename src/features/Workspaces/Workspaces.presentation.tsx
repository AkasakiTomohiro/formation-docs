import {
  Box,
  Button,
  ContentLayout,
  Flashbar,
  Header,
  Link,
  Modal,
  Pagination,
  SpaceBetween,
  Table,
} from '@cloudscape-design/components';
import type { useCollection } from '@cloudscape-design/collection-hooks';
import type { ButtonProps, FlashbarProps, LinkProps, ModalProps, TableProps } from '@cloudscape-design/components';
import type { WorkspaceExpand } from '../../hooks/useWorkspaces';

export type WorkspacesPresentationProps = {
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

  /**
   * モーダルの表示状態
   */
  isVisibleModal: boolean;

  /**
   * モーダルを閉じる際のハンドラー
   */
  onDismissModal: ModalProps['onDismiss'];

  /**
   * モーダルでキャンセルボタンクリック時のハンドラー
   */
  onClickCancelModal: ButtonProps['onClick'];

  /**
   * モーダルでリリースページへ遷移するボタンクリック時のハンドラー
   */
  onClickDownload: ButtonProps['onClick'];
};

export const WorkspacesPresentation = ({
  flashbarItems,
  isLoading,
  selectedItems,
  tableCollection,
  onClickWorkspaceLink,
  onClickNewWorkspace,
  onClickDeleteWorkspace,
  onClickUpdateWorkspace,
  onSelectionChange,
  isVisibleModal,
  onDismissModal,
  onClickCancelModal,
  onClickDownload,
}: WorkspacesPresentationProps): JSX.Element => {
  return (
    <ContentLayout
      defaultPadding
      header={
        <SpaceBetween size="m">
          <Flashbar items={flashbarItems} />
        </SpaceBetween>
      }
    >
      <Modal
        onDismiss={onDismissModal}
        visible={isVisibleModal}
        footer={
          <Box float="right">
            <SpaceBetween direction="horizontal" size="xs">
              <Button variant="link" onClick={onClickCancelModal}>
                このバージョンはスキップする
              </Button>
              <Button variant="primary" onClick={onClickDownload}>
                はい
              </Button>
            </SpaceBetween>
          </Box>
        }
        header="アプリを更新してください"
      >
        リリースページへ移動します。
      </Modal>
      <Table
        {...tableCollection.collectionProps}
        columnDefinitions={[
          {
            id: 'Name',
            header: 'ワークスペース名',
            cell: (e) => <Link onClick={onClickWorkspaceLink(e)}>{e.name}</Link>,
            isRowHeader: true,
          },
          {
            id: 'Directory',
            header: 'ディレクトリ',
            cell: (e) => e.directory,
          },
          {
            id: 'Description',
            header: '説明',
            cell: (e) => e.description,
          },
        ]}
        selectionType="single"
        selectedItems={selectedItems}
        onSelectionChange={onSelectionChange}
        items={tableCollection.items}
        loadingText="読み込み中..."
        trackBy="name"
        empty={
          <Box margin={{ vertical: 'xs' }} textAlign="center" color="inherit">
            <SpaceBetween size="m">
              <b>リソースがありません</b>
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
