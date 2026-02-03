import { Button, ContentLayout, Header, Input, SpaceBetween } from '@cloudscape-design/components';
import { PropertyTable } from '../../../PropertyTable';
import type { ButtonProps, TableProps } from '@cloudscape-design/components';
import type { ResourceTableItem } from '../../../../lib/CreateResourceTableItems';

export type ResourcePropertyTablePresentationProps = {
  /**
   * タブID
   */
  tabId: string;

  /**
   * 選択している論理ID
   */
  selectedLogicalId: string;

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
   */
  editingReasons?: Record<string, string>;

  /**
   * リソースの説明
   */
  description: string;

  /**
   * リソースの説明を設定する関数
   */
  setDescription: (description: string) => void;

  /**
   * ネストされた行の開閉関連イベント
   */
  expandedItems: TableProps<ResourceTableItem>['expandableRows'];

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
   * ネストされたプロパティの展開状態を更新する関数
   */
  setExpandedItems: (items: any) => void;
};

export const ResourcePropertyTablePresentation = ({
  tabId,
  selectedLogicalId,
  properties,
  expandedItems,
  onClickEdit,
  onClickCancel,
  onClickSave,
  reasons,
  setExpandedItems,
  editingReasons,
  description,
  setDescription,
}: ResourcePropertyTablePresentationProps): JSX.Element => {
  return (
    <ContentLayout
      defaultPadding
      header={
        <Header
          actions={
            <SpaceBetween direction="horizontal" size="xs">
              {editingReasons === undefined ? (
                <Button onClick={onClickEdit}>編集</Button>
              ) : (
                <>
                  <Button onClick={onClickCancel}>キャンセル</Button>
                  <Button variant="primary" onClick={onClickSave}>
                    保存
                  </Button>
                </>
              )}
            </SpaceBetween>
          }
          description={editingReasons === undefined ? description : undefined}
        >
          {selectedLogicalId}
        </Header>
      }
    >
      <SpaceBetween direction="vertical" size="m">
        {editingReasons !== undefined && (
          <Input
            onChange={({ detail }) => setDescription(detail.value)}
            value={description}
            placeholder="リソースの説明"
          />
        )}
        <PropertyTable
          tabId={tabId}
          properties={properties}
          reasons={reasons}
          editingReasons={editingReasons}
          expandedItems={expandedItems}
          setExpandedItems={setExpandedItems}
        />
      </SpaceBetween>
    </ContentLayout>
  );
};
