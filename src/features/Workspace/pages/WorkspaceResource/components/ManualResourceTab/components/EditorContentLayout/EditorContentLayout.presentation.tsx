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
import {
  Button,
  CodeEditor,
  Container,
  ContentLayout,
  Header,
  SegmentedControl,
  SpaceBetween,
} from '@cloudscape-design/components';

import { PropertyTable } from '../../../PropertyTable';

import type { ButtonProps, CodeEditorProps, SegmentedControlProps, TableProps } from '@cloudscape-design/components';

import type { ResourceTableItem } from '../../../../lib/CreateResourceTableItems';
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
   * リソースのプロパティ一覧
   */
  properties: ResourceTableItem[];

  /**
   * 編集中かどうか
   */
  isEdit: boolean;

  /**
   * コンテンツで表示する種別
   */
  viewMode: ViewMode;

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
  // biome-ignore lint/suspicious/noExplicitAny: <explanation>
  setExpandedItems: (items: any) => void;

  /**
   * 編集中の理由を更新
   */
  setEditingReasons: (reasons: (prev: Record<string, string>) => Record<string, string>) => void;

  /**
   * セグメントコントロールの変更イベントハンドラ
   */
  onChangeSegmentedControl: SegmentedControlProps['onChange'];

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

const selectModeOptions = [
  { text: 'Reason', id: 'reason' },
  { text: 'Value', id: 'value' },
];

export const EditorContentLayoutPresentation = ({
  selectedResourceId,
  properties,
  isEdit,
  viewMode,
  reasons,
  editingReasons,
  expandedItems,
  values,
  editingValues,
  acePreferences,
  onClickEdit,
  onClickCancel,
  onClickSave,
  setExpandedItems,
  setEditingReasons,
  onChangeSegmentedControl,
  onValidate,
  onPreferencesChange,
  onDelayedChange,
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
          properties={properties}
          reasons={reasons}
          expandedItems={expandedItems}
          setExpandedItems={setExpandedItems}
          editingReasons={editingReasons}
          setEditingReasons={setEditingReasons}
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
        <Container
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
        >
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
      )}
    </ContentLayout>
  );
};
