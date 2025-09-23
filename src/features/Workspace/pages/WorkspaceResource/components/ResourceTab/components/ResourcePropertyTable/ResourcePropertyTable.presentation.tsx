import { Button, ContentLayout, Header, SpaceBetween } from '@cloudscape-design/components';
import { PropertyTable } from '../../../PropertyTable';
import type { ButtonProps, TableProps } from '@cloudscape-design/components';
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

  /**
   * 編集中の理由を更新
   */
  setEditingReasons: (reasons: (prev: Record<string, string>) => Record<string, string>) => void;
};

export const ResourcePropertyTablePresentation = ({
  selectedLogicalId,
  properties,
  isEdit,
  expandedItems,
  onClickEdit,
  onClickCancel,
  onClickSave,
  reasons,
  setExpandedItems,
  editingReasons,
  setEditingReasons,
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
      <PropertyTable
        isEdit={isEdit}
        properties={properties}
        reasons={reasons}
        editingReasons={editingReasons}
        expandedItems={expandedItems}
        setExpandedItems={setExpandedItems}
        setEditingReasons={setEditingReasons}
      />
    </ContentLayout>
  );
};
