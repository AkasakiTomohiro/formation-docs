import {
  Box,
  CollectionPreferences,
  Link,
  StatusIndicator,
  Table,
  Textarea,
  TextFilter,
} from '@cloudscape-design/components';
import type { CollectionPreferencesProps, LinkProps, TableProps, TextareaProps } from '@cloudscape-design/components';
import type { WorkspaceTabInfo } from '../../../../contexts';
import type { ResourceTableItem } from '../../lib/CreateResourceTableItems';

export type PropertyTablePresentationProps = {
  /**
   * リソースのプロパティ一覧
   */
  properties: ResourceTableItem[];

  /**
   * Reasonの値
   */
  reasons: Record<string, string>;

  /**
   * 編集中のReasonの値
   * 編集中状態でなければ`undefined`
   */
  editingReasons?: Record<string, string>;

  /**
   * Reasonsのテキストエリアの変更イベントハンドラ
   */
  onChangeReason: (item: ResourceTableItem) => TextareaProps['onChange'];

  /**
   * Valueがリンクの場合のクリックイベントハンドラ
   */
  onClickValueLink: (item: WorkspaceTabInfo) => LinkProps['onClick'];

  /**
   * ネストされた行の開閉関連イベント
   */
  expandableRows: TableProps<ResourceTableItem>['expandableRows'];

  /**
   * 表示設定
   */
  preferences: CollectionPreferencesProps.Preferences<any>;

  /**
   * 表示設定変更時のイベントハンドラ
   */
  onConfirmPreferences: CollectionPreferencesProps['onConfirm'];

  /**
   * テーブルのヘッダー
   */
  header?: TableProps['header'];
};

export const PropertyTablePresentation = ({
  properties,
  expandableRows,
  reasons,
  editingReasons,
  onChangeReason,
  onClickValueLink,
  preferences,
  onConfirmPreferences,
  header,
}: PropertyTablePresentationProps): JSX.Element => {
  return (
    <Table
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
          id: 'type',
          header: '型',
          cell: (e) => <div style={{ whiteSpace: 'pre-line' }}>{e.type}</div>,
          width: 150,
          minWidth: 100,
        },
        {
          id: 'description',
          header: '説明',
          cell: (e) => <div style={{ whiteSpace: 'pre-line' }}>{e.description}</div>,
          width: 500,
        },
        {
          id: 'value',
          header: '値',
          cell: (e) => {
            const tableItemValue = e.value;
            if (tableItemValue === undefined) {
              return <div style={{ whiteSpace: 'pre-line' }} />;
            }
            if (tableItemValue.type === 'value') {
              return <div style={{ whiteSpace: 'pre-line' }}>{tableItemValue.value}</div>;
            }
            if (tableItemValue.type === 'array') {
              return (
                <ul style={{ listStyle: 'none', paddingLeft: 0, margin: 0 }}>
                  {tableItemValue.value.map((item, index) => {
                    if (item.type === 'value') {
                      return <li style={{ whiteSpace: 'pre-line' }}>{item.value}</li>;
                    }
                    const { value, ...other } = item;
                    return (
                      <li key={`${other.tabId}-${index}`}>
                        <Link onClick={onClickValueLink(other)}>{value}</Link>
                      </li>
                    );
                  })}
                </ul>
              );
            }
            const { value, ...other } = tableItemValue;
            return <Link onClick={onClickValueLink(other)}>{value}</Link>;
          },
          width: 300,
          minWidth: 100,
        },
        {
          id: 'reason',
          header: '設定理由',
          cell: (e) => {
            if (editingReasons !== undefined) {
              return <Textarea onChange={onChangeReason(e)} value={editingReasons?.[e.id] ?? ''} />;
            }
            return <div style={{ whiteSpace: 'pre-line' }}>{reasons[e.id]}</div>;
          },
          width: 300,
          minWidth: 200,
        },
      ]}
      columnDisplay={preferences.contentDisplay}
      stickyHeader
      enableKeyboardNavigation
      items={properties}
      loadingText="リソースを読み込み中..."
      trackBy="id"
      empty={
        <Box margin={{ vertical: 'xs' }} textAlign="center" color="inherit">
          <b>プロパティがありません</b>
        </Box>
      }
      filter={<TextFilter filteringPlaceholder="プロパティ検索" filteringText="" countText="0 件の一致" />}
      header={header}
      preferences={
        <CollectionPreferences
          title="設定"
          confirmLabel="確認"
          cancelLabel="キャンセル"
          preferences={preferences}
          onConfirm={onConfirmPreferences}
          contentDisplayPreference={{
            description: '列の表示／非表示や並び順をカスタマイズする',
            options: [
              {
                id: 'property',
                label: 'プロパティ',
                alwaysVisible: true,
              },
              { id: 'type', label: '型' },
              { id: 'description', label: '説明' },
              { id: 'value', label: '値' },
              { id: 'reason', label: '設定理由' },
            ],
          }}
        />
      }
    />
  );
};
