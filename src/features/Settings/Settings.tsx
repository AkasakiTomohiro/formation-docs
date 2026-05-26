import { useEffect, useState } from 'react';
import { useAppConfigContext } from '../../contexts/AppConfigContext';
import { useFlashbarContext } from '../../contexts/FlashbarContext';
import { ENGLISH_OPTION, JAPANESE_OPTION, SettingsPresentation } from './Settings.presentation';
import type { SelectProps } from '@cloudscape-design/components';
import type { Language } from '../../invoke/AppConfig';

export const Settings = () => {
  const { appConfig, setLanguage } = useAppConfigContext();
  const { addFlashbarItem } = useFlashbarContext();
  const [selectedLanguage, setSelectedLanguage] = useState<SelectProps['selectedOption']>(null);

  useEffect(() => {
    if (appConfig?.language === JAPANESE_OPTION.value) {
      setSelectedLanguage(JAPANESE_OPTION);
    } else {
      setSelectedLanguage(ENGLISH_OPTION);
    }
  }, [appConfig]);

  const handleChangeSelectedLanguage: SelectProps['onChange'] = (event) => {
    setSelectedLanguage(event.detail.selectedOption);
  };

  const handleClickSaveSelectedLanguage = async () => {
    if (selectedLanguage?.value !== undefined) {
      const result = await setLanguage(selectedLanguage.value as Language);
      if (result) {
        addFlashbarItem({
          type: 'success',
          header: '保存成功',
          content: '言語設定の保存に成功しました',
        });
      } else {
        addFlashbarItem({
          type: 'error',
          header: '保存失敗',
          content: '言語設定の保存に失敗しました',
        });
      }
    }
  };

  return (
    <SettingsPresentation
      selectedLanguage={selectedLanguage}
      onChangeSelectedLanguage={handleChangeSelectedLanguage}
      onClickSaveSelectedLanguage={handleClickSaveSelectedLanguage}
      services={[]}
      selectedService={null}
      onChangeService={() => {}}
      resources={[]}
      selectedResource={null}
      onChangeResource={() => {}}
      onClickResourceSelect={() => {}}
    />
  );
};
