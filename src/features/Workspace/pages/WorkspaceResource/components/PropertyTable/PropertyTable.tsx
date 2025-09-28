import { useState } from 'react';
import { useWorkspaceResourceContext } from '../../../../contexts';
import { PropertyTablePresentation } from './PropertyTable.presentation';
import type { ResourceTableItem } from '../../lib/CreateResourceTableItems';
import type { PropertyTablePresentationProps } from './PropertyTable.presentation';

export type PropertyTableProps = Pick<
  PropertyTablePresentationProps,
  'properties' | 'isEdit' | 'reasons' | 'editingReasons' | 'header'
> & {
  /**
   * ネストされたプロパティの展開状態
   */
  expandedItems: any;

  /**
   * ネストされたプロパティの展開状態を更新する関数
   */
  setExpandedItems: (items: any) => void;

  /**
   * 編集中の理由を更新
   */
  setEditingReasons: (reasons: (prev: Record<string, string>) => Record<string, string>) => void;
};

export const PropertyTable = ({
  properties,
  isEdit,
  reasons,
  expandedItems,
  setExpandedItems,
  editingReasons,
  setEditingReasons,
  header,
}: PropertyTableProps) => {
  // テーブルの表示カラムの設定を管理するためのステート
  const [preferences, setPreferences] = useState({
    contentDisplay: [
      { id: 'property', visible: true },
      { id: 'type', visible: true },
      { id: 'description', visible: true },
      { id: 'value', visible: true },
      { id: 'reason', visible: true },
    ],
  });
  const { addResourceTab } = useWorkspaceResourceContext();

  const onChangeReason: PropertyTablePresentationProps['onChangeReason'] =
    (item) =>
    ({ detail }) => {
      setEditingReasons((prev) => ({
        ...prev,
        [item.id]: detail.value,
      }));
    };

  return (
    <PropertyTablePresentation
      properties={properties}
      isEdit={isEdit}
      reasons={reasons}
      header={header}
      editingReasons={editingReasons}
      onChangeReason={onChangeReason}
      onClickValueLink={(item) => () => addResourceTab(item)}
      expandableRows={{
        getItemChildren: (item) => item.children ?? [],
        isItemExpandable: (item) => Boolean(item.children),
        expandedItems: expandedItems,
        onExpandableItemToggle: ({ detail }) =>
          setExpandedItems((prev: ResourceTableItem[] | undefined) => {
            const next = new Set((prev ?? []).map((item) => item.id));
            detail.expanded ? next.add(detail.item.id) : next.delete(detail.item.id);
            return [...next].map((id) => ({ id }));
          }),
      }}
      preferences={preferences}
      onConfirmPreferences={({ detail }) =>
        setPreferences({
          contentDisplay: detail.contentDisplay ? [...detail.contentDisplay] : [],
        })
      }
    />
  );
};

/**
 * リソーステーブル用の展開済みアイテムを作成する
 * @param items
 * @returns
 */
export function createExpandedItems(items: ResourceTableItem[]): ResourceTableItem[] {
  const expandedItems: ResourceTableItem[] = [];
  for (const item of items) {
    if (item.children !== undefined) {
      expandedItems.push(item);
      expandedItems.push(...createExpandedItems(item.children));
    }
  }
  return expandedItems;
}
