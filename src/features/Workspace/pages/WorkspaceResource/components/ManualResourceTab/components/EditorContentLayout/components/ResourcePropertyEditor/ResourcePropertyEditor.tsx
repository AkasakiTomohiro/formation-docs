import { useState } from 'react';

import { ResourcePropertyEditorPresentation } from './ResourcePropertyEditor.presentation';

import type { ResourcePropertyEditorPresentationProps } from './ResourcePropertyEditor.presentation';

import type { CodeEditorProps } from '@cloudscape-design/components';

export type ResourcePropertyEditorProps = Omit<
  ResourcePropertyEditorPresentationProps,
  'acePreferences' | 'onPreferencesChange'
>;

export const ResourcePropertyEditor = ({
  isEdit,
  values,
  editingValues,
  header,
  onValidate,
  onDelayedChange,
}: ResourcePropertyEditorProps): JSX.Element => {
  // biome-ignore lint/suspicious/noExplicitAny: <explanation>
  const [acePreferences, setAcePreferences] = useState<CodeEditorProps.Preferences>({} as any);

  return (
    <ResourcePropertyEditorPresentation
      isEdit={isEdit}
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
