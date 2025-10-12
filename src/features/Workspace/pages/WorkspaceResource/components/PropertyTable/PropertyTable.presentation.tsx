import {
  Box,
  Button,
  CollectionPreferences,
  Link,
  SpaceBetween,
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
      renderAriaLive={({ firstIndex, lastIndex, totalItemsCount }) =>
        `Displaying items ${firstIndex} to ${lastIndex} of ${totalItemsCount}`
      }
      renderLoaderPending={() => (
        <Button variant="inline-link" iconName="add-plus">
          Show more
        </Button>
      )}
      renderLoaderLoading={() => <StatusIndicator type="loading">Loading items</StatusIndicator>}
      renderLoaderError={() => <StatusIndicator type="error">Loading error</StatusIndicator>}
      renderLoaderEmpty={() => <Box>No resources found</Box>}
      expandableRows={expandableRows}
      resizableColumns
      columnDefinitions={[
        {
          id: 'property',
          header: 'Property',
          cell: (e) => e.property,
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
        {
          id: 'value',
          header: 'Value',
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
          header: 'Reason',
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
      loadingText="Loading resources"
      trackBy="id"
      empty={
        <Box margin={{ vertical: 'xs' }} textAlign="center" color="inherit">
          <SpaceBetween size="m">
            <b>No resources</b>
            <Button>Create resource</Button>
          </SpaceBetween>
        </Box>
      }
      filter={<TextFilter filteringPlaceholder="Find resources" filteringText="" countText="0 matches" />}
      header={header}
      preferences={
        <CollectionPreferences
          title="Preferences"
          confirmLabel="Confirm"
          cancelLabel="Cancel"
          preferences={preferences}
          onConfirm={onConfirmPreferences}
          contentDisplayPreference={{
            description: 'Customize the visibility and order of the columns.',
            options: [
              {
                id: 'property',
                label: 'Property',
                alwaysVisible: true,
              },
              { id: 'type', label: 'Type' },
              { id: 'description', label: 'Description' },
              { id: 'value', label: 'Value' },
              { id: 'reason', label: 'Reason' },
            ],
          }}
        />
      }
    />
  );
};
