import { useCollection } from '@cloudscape-design/collection-hooks';
import {
  Box,
  Button,
  ContentLayout,
  Header,
  SpaceBetween,
  StatusIndicator,
  Table,
  type TableProps,
  Textarea,
  type TextareaProps,
  TextFilter,
} from '@cloudscape-design/components';
import type { ResourceTableItem } from '../../../Workspace/pages/WorkspaceResource/lib/CreateResourceTableItems';

export type TableItem = ResourceTableItem;

export type DescriptionTranslatePresentationProps = {
  /**
   * サービス名
   */
  serviceName: string;

  /**
   * リソースタイプ
   */
  resourceType: string;

  /**
   * プロパティ
   */
  properties: TableItem[];

  /**
   * 編集中のTranslationの値
   * 編集中状態でなければ`undefined`
   */
  editingTranslations?: Record<string, string>;

  /**
   * Translationsのテキストエリアの変更イベントハンドラ
   */
  onChangeTranslation: (item: TableItem) => TextareaProps['onChange'];

  /**
   * ネストされた行の開閉関連イベント
   */
  expandableRows: TableProps<TableItem>['expandableRows'];

  /**
   * 保存ボタンのクリックイベントハンドラ
   */
  onClickSave: () => void;

  /**
   * キャンセルボタンのクリックイベントハンドラ
   */
  onClickCancel: () => void;
};

export const DescriptionTranslatePresentation = ({
  properties,
  expandableRows,
  editingTranslations,
  onChangeTranslation,
  onClickSave,
  onClickCancel,
  serviceName,
  resourceType,
}: DescriptionTranslatePresentationProps) => {
  const { items, collectionProps, filterProps } = useCollection<TableItem>(properties, { filtering: {} });
  return (
    <ContentLayout
      defaultPadding
      header={
        <Header
          variant="h2"
          actions={
            <SpaceBetween direction="horizontal" size="xs">
              <Button variant="normal" onClick={onClickCancel}>
                キャンセル
              </Button>
              <Button variant="primary" onClick={onClickSave}>
                保存
              </Button>
            </SpaceBetween>
          }
        >
          リソースのプロパティ翻訳
        </Header>
      }
    >
      <Table
        {...collectionProps}
        renderLoaderLoading={() => <StatusIndicator type="loading">読み込み中</StatusIndicator>}
        renderLoaderError={() => <StatusIndicator type="error">読み込みエラー</StatusIndicator>}
        renderLoaderEmpty={() => <Box>プロパティが見つかりません</Box>}
        expandableRows={expandableRows}
        resizableColumns
        columnDefinitions={[
          {
            id: 'property',
            header: 'プロパティ',
            cell: (e) => e.property,
            isRowHeader: true,
            width: 250,
            minWidth: 150,
          },
          {
            id: 'description',
            header: '説明',
            cell: (e) => <div style={{ whiteSpace: 'pre-line' }}>{e.description}</div>,
            width: 500,
          },
          {
            id: 'translation',
            header: '翻訳',
            cell: (e) => <Textarea onChange={onChangeTranslation(e)} value={editingTranslations?.[e.id] ?? ''} />,
            width: 300,
            minWidth: 200,
          },
        ]}
        stickyHeader
        enableKeyboardNavigation
        items={items}
        loadingText="読み込み中..."
        trackBy="id"
        empty={
          <Box margin={{ vertical: 'xs' }} textAlign="center" color="inherit">
            <b>プロパティがありません</b>
          </Box>
        }
        filter={<TextFilter {...filterProps} filteringPlaceholder="検索" />}
        header={
          <Header variant="h3">
            {serviceName}::{resourceType}
          </Header>
        }
      />
    </ContentLayout>
  );
};
