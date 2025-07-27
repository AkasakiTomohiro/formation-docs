import {
  Box,
  Button,
  CollectionPreferences,
  Header,
  SpaceBetween,
  StatusIndicator,
  Table,
  TextFilter,
  Textarea,
} from '@cloudscape-design/components';

import type { ResourceTableItem } from '../../lib/CreateResourceTableItems';

import type { CollectionPreferencesProps, TableProps, TextareaProps } from '@cloudscape-design/components';
export type PropertyTablePresentationProps = {
  /**
   * リソースのプロパティ一覧
   */
  properties: ResourceTableItem[];

  /**
   * 編集中かどうか
   */
  isEdit: boolean;

  /**
   * Reasonの値
   */
  reasons: Record<string, string>;

  /**
   * 編集中のReasonの値
   */
  editingReasons: Record<string, string>;

  /**
   * Reasonsのテキストエリアの変更イベントハンドラ
   */
  onChangeReason: (item: ResourceTableItem) => TextareaProps['onChange'];

  /**
   * ネストされた行の開閉関連イベント
   */
  expandableRows: TableProps<ResourceTableItem>['expandableRows'];

  /**
   * 表示設定
   */
  // biome-ignore lint/suspicious/noExplicitAny: <explanation>
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
  isEdit,
  expandableRows,
  reasons,
  editingReasons,
  onChangeReason,
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
          cell: (e) => <div style={{ whiteSpace: 'pre-line' }}>{e.value === undefined ? '' : `${e.value}`}</div>,
          width: 300,
          minWidth: 100,
        },
        {
          id: 'reason',
          header: 'Reason',
          cell: (e) => {
            if (isEdit) {
              return <Textarea onChange={onChangeReason(e)} value={editingReasons[e.id] ?? ''} />;
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
