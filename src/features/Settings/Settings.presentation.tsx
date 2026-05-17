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
};

export const SettingsPresentation = ({
  selectedLanguage,
  onChangeSelectedLanguage,
  services,
  selectedService,
  onChangeService,
  resources,
  selectedResource,
  onChangeResource,
  onClickResourceSelect,
}: SettingsPresentationProps) => {
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
          <div style={{ width: 300 }}>
            <Select
              selectedOption={selectedLanguage}
              onChange={onChangeSelectedLanguage}
              options={[
                { label: '日本語', value: 'ja' },
                { label: 'English', value: 'en' },
              ]}
            />
          </div>
        </Container>

        <Container
          header={
            <Header variant="h2" description="プロパティの説明文をカスタマイズできます">
              プロパティの説明文のカスタマイズ
            </Header>
          }
        >
          <SpaceBetween direction="vertical" size="s">
            <Select
              placeholder="サービス名"
              selectedOption={selectedService}
              onChange={onChangeService}
              options={services}
              filteringType="auto"
            />
            <Select
              placeholder="リソース名"
              disabled={!selectedService}
              selectedOption={selectedResource}
              onChange={onChangeResource}
              options={resources}
              filteringType="auto"
            />
            <Box float="right">
              <Button variant="primary" onClick={onClickResourceSelect}>
                選択
              </Button>
            </Box>
          </SpaceBetween>
        </Container>
      </SpaceBetween>
    </ContentLayout>
  );
};
