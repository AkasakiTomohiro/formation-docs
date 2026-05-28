import { useEffect, useState } from 'react';
import { useAppConfigContext } from '../../contexts/AppConfigContext';
import { useFlashbarContext } from '../../contexts/FlashbarContext';
import { getAWSServiceList } from '../../invoke/GetAWSServiceList';
import { ENGLISH_OPTION, JAPANESE_OPTION, SettingsPresentation } from './Settings.presentation';
import type { SelectProps } from '@cloudscape-design/components';
import type { Language } from '../../invoke/AppConfig';

export type AWSService = {
  service_name: string;
  resources: string[];
};

export const Settings = () => {
  const { appConfig, setLanguage } = useAppConfigContext();
  const { addFlashbarItem } = useFlashbarContext();
  const [selectedLanguage, setSelectedLanguage] = useState<SelectProps['selectedOption']>(null);
  const [selectedService, setSelectedService] = useState<SelectProps['selectedOption']>(null);
  const [services, setServices] = useState<AWSService[]>([]);
  const [selectedResource, setSelectedResource] = useState<SelectProps['selectedOption']>(null);

  useEffect(() => {
    if (appConfig?.language === JAPANESE_OPTION.value) {
      setSelectedLanguage(JAPANESE_OPTION);
    } else {
      setSelectedLanguage(ENGLISH_OPTION);
    }

    getAWSServiceList().then((serviceList) => {
      setServices(
        Object.entries(serviceList)
          .sort((a, b) => a[0].localeCompare(b[0]))
          .map((m) => ({
            service_name: m[0],
            resources: m[1].sort((x, y) => x.localeCompare(y)),
          })),
      );
    });
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

  const handleChangeSelectedService: SelectProps['onChange'] = ({ detail }) => {
    if (selectedService?.value !== detail.selectedOption) {
      setSelectedResource(null);
    }
    setSelectedService(detail.selectedOption);
  };

  const handleChangeSelectedResource: SelectProps['onChange'] = (event) => {
    setSelectedResource(event.detail.selectedOption);
  };

  return (
    <SettingsPresentation
      selectedLanguage={selectedLanguage}
      onChangeSelectedLanguage={handleChangeSelectedLanguage}
      onClickSaveSelectedLanguage={handleClickSaveSelectedLanguage}
      services={services.map((item) => ({ value: item.service_name }))}
      selectedService={selectedService}
      onChangeService={handleChangeSelectedService}
      resources={services
        .find((service) => service.service_name === selectedService?.value)
        ?.resources.map((resource) => ({ value: resource }))}
      selectedResource={selectedResource}
      onChangeResource={handleChangeSelectedResource}
      onClickResourceSelect={() => {}}
    />
  );
};
