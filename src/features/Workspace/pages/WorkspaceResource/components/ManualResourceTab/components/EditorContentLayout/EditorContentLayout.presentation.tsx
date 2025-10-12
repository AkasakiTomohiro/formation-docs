import { Button, ContentLayout, Header, SegmentedControl, SpaceBetween } from '@cloudscape-design/components';
import { PropertyTable } from '../../../PropertyTable';
import { ResourcePropertyEditor } from './components';
import type { ButtonProps, SegmentedControlProps } from '@cloudscape-design/components';
import type { ViewMode } from '../../../../../../contexts';
import type { PropertyTableProps } from '../../../PropertyTable';
import type { ResourcePropertyEditorProps } from './components';

export type EditorContentLayoutPresentationProps = {
  /**
   * タブID
   */
  tabId: string;

  /**
   * 選択しているリソースID
   */
  selectedResourceId: string;

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
  propertyTableProps: Omit<PropertyTableProps, 'header'>;

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
  tabId,
  selectedResourceId,
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
              {propertyTableProps.editingReasons === undefined ? (
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
        >
          {selectedResourceId}
        </Header>
      }
    >
      {viewMode === 'reason' && (
        <PropertyTable
          tabId={tabId}
          properties={propertyTableProps.properties}
          reasons={propertyTableProps.reasons}
          expandedItems={propertyTableProps.expandedItems}
          setExpandedItems={propertyTableProps.setExpandedItems}
          editingReasons={propertyTableProps.editingReasons}
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
