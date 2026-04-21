import { useState } from 'react';
import { ResourceDescriptionSettingsPresentation } from './ResourceDescriptionSettings.presentation';
import type { SelectProps } from '@cloudscape-design/components';

export const ResourceDescriptionSettings = () => {
  const [selectedLanguage, setSelectedLanguage] = useState<SelectProps['selectedOption']>(null);

  const handleChangeSelectedLanguage: SelectProps['onChange'] = (event) => {
    setSelectedLanguage(event.detail.selectedOption);
  };

  return (
    <ResourceDescriptionSettingsPresentation
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
