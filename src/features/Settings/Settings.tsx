import { useState } from 'react';
import { SettingsPresentation } from './Settings.presentation';
import type { SelectProps } from '@cloudscape-design/components';

export const Settings = () => {
  const [selectedLanguage, setSelectedLanguage] = useState<SelectProps['selectedOption']>(null);

  const handleChangeSelectedLanguage: SelectProps['onChange'] = (event) => {
    setSelectedLanguage(event.detail.selectedOption);
  };

  return (
    <SettingsPresentation
      selectedLanguage={selectedLanguage}
      onChangeSelectedLanguage={handleChangeSelectedLanguage}
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
