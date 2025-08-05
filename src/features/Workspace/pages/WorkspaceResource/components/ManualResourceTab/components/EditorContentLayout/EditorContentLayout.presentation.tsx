import { ContentLayout } from '@cloudscape-design/components';

import { PropertyTable } from '../../../PropertyTable';
import { ResourcePropertyEditor } from './components';
import { TableHeader } from './components';

import type { PropertyTableProps } from '../../../PropertyTable';
import type { ResourcePropertyEditorProps } from './components';

import type { TableHeaderProps } from './components';

export type EditorContentLayoutPresentationProps = TableHeaderProps & {
  /**
   * PropertyTableコンポーネントのprops
   */
  propertyTableProps: Omit<PropertyTableProps, 'isEdit' | 'header'>;

  /**
   * ResourcePropertyEditorコンポーネントのprops
   */
  resourcePropertyEditorProps: Omit<ResourcePropertyEditorProps, 'isEdit' | 'header'>;
};

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
    <ContentLayout>
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
            <TableHeader
              selectedResourceId={selectedResourceId}
              isEdit={isEdit}
              onClickEdit={onClickEdit}
              onClickCancel={onClickCancel}
              onClickSave={onClickSave}
              viewMode={viewMode}
              onChangeSegmentedControl={onChangeSegmentedControl}
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
            <TableHeader
              selectedResourceId={selectedResourceId}
              isEdit={isEdit}
              onClickEdit={onClickEdit}
              onClickCancel={onClickCancel}
              onClickSave={onClickSave}
              viewMode={viewMode}
              onChangeSegmentedControl={onChangeSegmentedControl}
            />
          }
        />
      )}
    </ContentLayout>
  );
};
