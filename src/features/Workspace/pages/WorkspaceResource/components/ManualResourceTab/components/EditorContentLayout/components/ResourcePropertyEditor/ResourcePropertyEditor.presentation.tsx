import ace from 'ace-builds/src-noconflict/ace';

import 'ace-builds/css/ace.css';
import 'ace-builds/esm-resolver';
import 'ace-builds/src-noconflict/ext-language_tools';
import 'ace-builds/src-noconflict/mode-json';
import 'ace-builds/src-noconflict/snippets/json';
import 'ace-builds/src-noconflict/theme-github';
import jsonWorker from 'ace-builds/src-noconflict/worker-json?url';

import CodeView from '@cloudscape-design/code-view/code-view';
import jsonHighlight from '@cloudscape-design/code-view/highlight/json';
import { CodeEditor, Container } from '@cloudscape-design/components';

import type { CodeEditorProps } from '@cloudscape-design/components';

ace.config.setModuleUrl('ace/mode/json_worker', jsonWorker);

const i18nStrings = {
  loadingState: 'Loading code editor',
  errorState: 'There was an error loading the code editor.',
  errorStateRecovery: 'Retry',
  editorGroupAriaLabel: 'Code editor',
  statusBarGroupAriaLabel: 'Status bar',
  cursorPosition: (row: number, column: number) => `Ln ${row}, Col ${column}`,
  errorsTab: 'Errors',
  warningsTab: 'Warnings',
  preferencesButtonAriaLabel: 'Preferences',
  paneCloseButtonAriaLabel: 'Close',
  preferencesModalHeader: 'Preferences',
  preferencesModalCancel: 'Cancel',
  preferencesModalConfirm: 'Confirm',
  preferencesModalWrapLines: 'Wrap lines',
  preferencesModalTheme: 'Theme',
  preferencesModalLightThemes: 'Light themes',
  preferencesModalDarkThemes: 'Dark themes',
};

export type ResourcePropertyEditorPresentationProps = {
  /**
   * 編集中かどうか
   */
  isEdit: boolean;

  /**
   * コンテナーヘッダーコンポーネント
   */
  header?: React.ReactNode;

  /**
   * Valueの値
   */
  values: string;

  /**
   * 編集中のValueの値
   */
  editingValues: string;

  /**
   * Aceエディタの表示設定
   */
  acePreferences: CodeEditorProps.Preferences;

  /**
   * Aceエディタのバリデーションイベントハンドラ
   */
  onValidate: CodeEditorProps['onValidate'];

  /**
   * Aceエディタの表示設定変更イベントハンドラ
   */
  onPreferencesChange: CodeEditorProps['onPreferencesChange'];

  /**
   * Aceエディタの遅延変更イベントハンドラ
   */
  onDelayedChange: CodeEditorProps['onDelayedChange'];
};

export const ResourcePropertyEditorPresentation = ({
  isEdit,
  header,
  values,
  editingValues,
  acePreferences,
  onValidate,
  onPreferencesChange,
  onDelayedChange,
}: ResourcePropertyEditorPresentationProps): JSX.Element => {
  return (
    <Container header={header}>
      {isEdit ? (
        <CodeEditor
          ace={ace}
          language="json"
          themes={{ light: ['github'], dark: ['github'] }}
          value={editingValues}
          onValidate={onValidate}
          preferences={acePreferences}
          onPreferencesChange={onPreferencesChange}
          onDelayedChange={onDelayedChange}
          i18nStrings={i18nStrings}
        />
      ) : (
        <CodeView content={values} lineNumbers highlight={jsonHighlight} />
      )}
    </Container>
  );
};
