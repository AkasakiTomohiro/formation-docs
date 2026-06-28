import { Button, ContentLayout, Header, Input, SegmentedControl, SpaceBetween } from '@cloudscape-design/components';
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
   * リソースの説明
   */
  description: string;

  /**
   * リソースの説明（編集中の値）
   */
  editingDescription: string;

  /**
   * リソースの説明を設定する関数
   */
  setDescription: (description: string) => void;

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
  description,
  editingDescription,
  setDescription,
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
          description={propertyTableProps.editingReasons === undefined ? description : undefined}
        >
          {selectedResourceId}
        </Header>
      }
    >
      <SpaceBetween direction="vertical" size="m">
        {propertyTableProps.editingReasons !== undefined && (
          <Input
            onChange={({ detail }) => setDescription(detail.value)}
            value={editingDescription}
            placeholder="リソースの説明"
          />
        )}
        {viewMode === 'reason' && (
          <PropertyTable
            tabId={tabId}
            properties={propertyTableProps.properties}
            reasons={propertyTableProps.reasons}
            expandedItems={propertyTableProps.expandedItems}
            setExpandedItems={propertyTableProps.setExpandedItems}
            editingReasons={propertyTableProps.editingReasons}
            translation={propertyTableProps.translation}
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
      </SpaceBetween>
    </ContentLayout>
  );
};
