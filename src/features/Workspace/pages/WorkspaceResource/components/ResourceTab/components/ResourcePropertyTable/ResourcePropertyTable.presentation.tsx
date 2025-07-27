import {
  Box,
  Button,
  CollectionPreferences,
  ContentLayout,
  Header,
  SpaceBetween,
  StatusIndicator,
  Table,
  TextFilter,
  Textarea,
} from '@cloudscape-design/components';

import type { ButtonProps, CollectionPreferencesProps, TableProps, TextareaProps } from '@cloudscape-design/components';
import type { ResourceTableItem } from '../../../../lib/CreateResourceTableItems';

export type ResourcePropertyTablePresentationProps = {
  /**
   * 選択している論理ID
   */
  selectedLogicalId: string;

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
   * 編集ボタン押下時のイベントハンドラ
   */
  onClickEdit: ButtonProps['onClick'];

  /**
   * キャンセルボタン押下時のイベントハンドラ
   */
  onClickCancel: ButtonProps['onClick'];

  /**
   * 保存ボタン押下時のイベントハンドラ
   */
  onClickSave: ButtonProps['onClick'];

  /**
   * 表示設定
   */

  // biome-ignore lint/suspicious/noExplicitAny: <explanation>
  preferences: CollectionPreferencesProps.Preferences<any>;

  /**
   * 表示設定変更時のイベントハンドラ
   */
  onConfirmPreferences: CollectionPreferencesProps['onConfirm'];
};

export const ResourcePropertyTablePresentation = ({
  selectedLogicalId,
  properties,
  isEdit,
  expandableRows,
  onClickEdit,
  onClickCancel,
  onClickSave,
  reasons,
  editingReasons,
  onChangeReason,
  preferences,
  onConfirmPreferences,
}: ResourcePropertyTablePresentationProps): JSX.Element => {
  return (
    <ContentLayout
      defaultPadding
      header={
        <Header
          actions={
            <SpaceBetween direction="horizontal" size="xs">
              {!isEdit && <Button onClick={onClickEdit}>編集</Button>}
              {isEdit && (
                <>
                  <Button onClick={onClickCancel}>キャンセル</Button>
                  <Button variant="primary" onClick={onClickSave}>
                    保存
                  </Button>
                </>
              )}
            </SpaceBetween>
          }
        >
          {selectedLogicalId}
        </Header>
      }
    >
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
        header={<Header>Table with expandable rows</Header>}
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
    </ContentLayout>
  );
};
