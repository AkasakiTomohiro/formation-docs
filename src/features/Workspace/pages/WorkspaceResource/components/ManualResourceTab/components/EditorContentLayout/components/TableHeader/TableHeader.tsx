import { Button, Header, SegmentedControl, SpaceBetween } from '@cloudscape-design/components';

import type { ButtonProps, SegmentedControlProps } from '@cloudscape-design/components';

/**
 * コンテンツで表示する種別
 */
export type ViewMode = 'reason' | 'value';

export type TableHeaderProps = {
  /**
   * 選択しているリソースID
   */
  selectedResourceId: string;

  /**
   * 編集中かどうか
   */
  isEdit: boolean;
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
   * コンテンツで表示する種別
   */
  viewMode: ViewMode;

  /**
   * セグメントコントロールの変更イベントハンドラ
   */
  onChangeSegmentedControl: SegmentedControlProps['onChange'];
};

const selectModeOptions = [
  { text: 'Reason', id: 'reason' },
  { text: 'Value', id: 'value' },
];

export const TableHeader = ({
  selectedResourceId,
  isEdit,
  onClickEdit,
  onClickCancel,
  onClickSave,
  viewMode,
  onChangeSegmentedControl,
}: TableHeaderProps): JSX.Element => {
  return (
    <Header
      actions={
        <SpaceBetween direction="horizontal" size="xs">
          <SegmentedControl selectedId={viewMode} onChange={onChangeSegmentedControl} options={selectModeOptions} />
          {!isEdit && <Button onClick={onClickEdit}>編集</Button>}
          {isEdit && <Button onClick={onClickCancel}>キャンセル</Button>}
          {isEdit && (
            <Button variant="primary" onClick={onClickSave}>
              保存
            </Button>
          )}
        </SpaceBetween>
      }
    >
      {selectedResourceId}
    </Header>
  );
};
