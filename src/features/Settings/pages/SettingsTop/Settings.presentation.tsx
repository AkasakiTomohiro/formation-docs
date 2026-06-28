import {
  Box,
  Button,
  Container,
  ContentLayout,
  Header,
  Select,
  type SelectProps,
  SpaceBetween,
} from '@cloudscape-design/components';
import { useAppConfigContext } from '../../../../contexts/AppConfigContext';

export type SettingsPresentationProps = {
  /**
   * 選択中の優先言語
   */
  selectedLanguage: SelectProps['selectedOption'];

  /**
   * 優先言語の変更イベントハンドラ
   */
  onChangeSelectedLanguage: SelectProps['onChange'];

  /**
   * 優先言語保存のイベントハンドラ
   */
  onClickSaveSelectedLanguage: () => void;

  /**
   * サービスのリスト
   */
  services: SelectProps['options'];

  /**
   * 選択中のサービス
   */
  selectedService: SelectProps['selectedOption'];

  /**
   * サービス選択の変更イベントハンドラ
   */
  onChangeService: SelectProps['onChange'];

  /**
   * リソースのリスト
   */
  resources: SelectProps['options'];

  /**
   * 選択中のリソース
   */
  selectedResource: SelectProps['selectedOption'];

  /**
   * リソース選択の変更イベントハンドラ
   */
  onChangeResource: SelectProps['onChange'];

  /**
   * リソース選択ボタンのクリックイベントハンドラ
   */
  onClickResourceSelect: () => void;

  /**
   * インポートボタンのクリックイベントハンドラ
   */
  onClickImport: () => void;

  /**
   * エクスポートボタンのクリックイベントハンドラ
   */
  onClickExport: () => void;
};

export const ENGLISH_OPTION = { label: 'English', value: 'En' } satisfies SelectProps['selectedOption'];
export const JAPANESE_OPTION = { label: '日本語', value: 'Ja' } satisfies SelectProps['selectedOption'];
const LANGUAGE_OPTIONS = [ENGLISH_OPTION, JAPANESE_OPTION] satisfies SelectProps['options'];

export const SettingsPresentation = ({
  selectedLanguage,
  onChangeSelectedLanguage,
  onClickSaveSelectedLanguage,
  services,
  selectedService,
  onChangeService,
  resources,
  selectedResource,
  onChangeResource,
  onClickResourceSelect,
  onClickImport,
  onClickExport,
}: SettingsPresentationProps) => {
  const { appConfig } = useAppConfigContext();

  return (
    <ContentLayout defaultPadding header={<Header variant="h1">設定</Header>}>
      <SpaceBetween size="l">
        <Container
          header={
            <Header variant="h2" description="プロパティの説明の言語を設定できます">
              優先言語
            </Header>
          }
        >
          <SpaceBetween direction="vertical" size="s">
            <div style={{ width: 300 }}>
              <Select
                selectedOption={selectedLanguage}
                onChange={onChangeSelectedLanguage}
                options={LANGUAGE_OPTIONS}
              />
            </div>
            <Box float="right">
              <Button
                variant="primary"
                onClick={onClickSaveSelectedLanguage}
                disabled={appConfig?.language === selectedLanguage?.value || appConfig?.language === undefined}
              >
                保存
              </Button>
            </Box>
          </SpaceBetween>
        </Container>

        <Container
          header={
            <Header
              variant="h2"
              description="プロパティの説明文をカスタマイズできます"
              actions={
                <SpaceBetween direction="horizontal" size="xs">
                  <Button
                    onClick={onClickImport}
                    disabled={appConfig?.language === ENGLISH_OPTION.value || appConfig?.language === undefined}
                  >
                    インポート
                  </Button>
                  <Button
                    onClick={onClickExport}
                    disabled={appConfig?.language === ENGLISH_OPTION.value || appConfig?.language === undefined}
                  >
                    エクスポート
                  </Button>
                </SpaceBetween>
              }
            >
              プロパティの説明文のカスタマイズ
            </Header>
          }
        >
          <SpaceBetween direction="vertical" size="s">
            <Header variant="h3">手動入力</Header>
            <Select
              placeholder="サービス名"
              disabled={appConfig?.language === ENGLISH_OPTION.value || appConfig?.language === undefined}
              selectedOption={selectedService}
              onChange={onChangeService}
              options={services}
              filteringType="auto"
            />
            <Select
              placeholder="リソース名"
              disabled={
                !selectedService || appConfig?.language === ENGLISH_OPTION.value || appConfig?.language === undefined
              }
              selectedOption={selectedResource}
              onChange={onChangeResource}
              options={resources}
              filteringType="auto"
            />
            <Box float="right">
              <Button
                variant="primary"
                disabled={
                  !selectedResource || appConfig?.language === ENGLISH_OPTION.value || appConfig?.language === undefined
                }
                onClick={onClickResourceSelect}
              >
                選択
              </Button>
            </Box>
          </SpaceBetween>
        </Container>
      </SpaceBetween>
    </ContentLayout>
  );
};
