import { useState } from 'react';
import { ResourcePropertyEditorPresentation } from './ResourcePropertyEditor.presentation';
import type { CodeEditorProps } from '@cloudscape-design/components';
import type { ResourcePropertyEditorPresentationProps } from './ResourcePropertyEditor.presentation';

export type ResourcePropertyEditorProps = Omit<
  ResourcePropertyEditorPresentationProps,
  'acePreferences' | 'onPreferencesChange'
>;

export const ResourcePropertyEditor = ({
  values,
  editingValues,
  header,
  onValidate,
  onDelayedChange,
}: ResourcePropertyEditorProps): JSX.Element => {
  const [acePreferences, setAcePreferences] = useState<CodeEditorProps.Preferences>({} as any);

  return (
    <ResourcePropertyEditorPresentation
      values={values}
      editingValues={editingValues}
      header={header}
      acePreferences={acePreferences}
      onPreferencesChange={(event) => setAcePreferences(event.detail)}
      onDelayedChange={onDelayedChange}
      onValidate={onValidate}
    />
  );
};
