import { Button, ContentLayout, Header, SegmentedControl, SpaceBetween } from '@cloudscape-design/components';
import { PropertyTable } from '../../../PropertyTable';
import { ResourcePropertyEditor } from './components';
import type { ButtonProps, SegmentedControlProps } from '@cloudscape-design/components';
import type { PropertyTableProps } from '../../../PropertyTable';
import type { ResourcePropertyEditorProps } from './components';

/**
 * コンテンツで表示する種別
 */
export type ViewMode = 'reason' | 'value';

export type EditorContentLayoutPresentationProps = {
  /**
   * 選択しているリソースID
   */
  selectedResourceId: string;

  /**
   * 編集中かどうか
   */
  isEdit: boolean;

  /**
   * コンテンツで表示する種別
   */
  viewMode: ViewMode;

  /**
   * セグメントコントロールの変更イベントハンドラ
   */
  onChangeSegmentedControl: SegmentedControlProps['onChange'];

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
   * PropertyTableコンポーネントのprops
   */
  propertyTableProps: Omit<PropertyTableProps, 'isEdit' | 'header'>;

  /**
   * ResourcePropertyEditorコンポーネントのprops
   */
  resourcePropertyEditorProps: Omit<ResourcePropertyEditorProps, 'isEdit' | 'header'>;
};

const selectModeOptions = [
  { text: 'Reason', id: 'reason' },
  { text: 'Value', id: 'value' },
];

export const EditorContentLayoutPresentation = ({
  selectedResourceId,
  isEdit,
  propertyTableProps,
  viewMode,
  onClickEdit,
  onClickCancel,
  onClickSave,
  onChangeSegmentedControl,
  resourcePropertyEditorProps,
}: EditorContentLayoutPresentationProps) => {
  return (
    <ContentLayout
      defaultPadding
      header={
        <Header
          actions={
            <SpaceBetween direction="horizontal" size="xs">
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
      }
    >
      {viewMode === 'reason' && (
        <PropertyTable
          isEdit={isEdit}
          properties={propertyTableProps.properties}
          reasons={propertyTableProps.reasons}
          expandedItems={propertyTableProps.expandedItems}
          setExpandedItems={propertyTableProps.setExpandedItems}
          editingReasons={propertyTableProps.editingReasons}
          setEditingReasons={propertyTableProps.setEditingReasons}
          header={
            <Header
              variant="h2"
              actions={
                <SegmentedControl
                  selectedId={viewMode}
                  onChange={onChangeSegmentedControl}
                  options={selectModeOptions}
                />
              }
            />
          }
        />
      )}
      {viewMode === 'value' && (
        <ResourcePropertyEditor
          isEdit={isEdit}
          values={resourcePropertyEditorProps.values}
          editingValues={resourcePropertyEditorProps.editingValues}
          onDelayedChange={resourcePropertyEditorProps.onDelayedChange}
          onValidate={resourcePropertyEditorProps.onValidate}
          header={
            <Header
              variant="h2"
              actions={
                <SegmentedControl
                  selectedId={viewMode}
                  onChange={onChangeSegmentedControl}
                  options={selectModeOptions}
                />
              }
            />
          }
        />
      )}
    </ContentLayout>
  );
};
