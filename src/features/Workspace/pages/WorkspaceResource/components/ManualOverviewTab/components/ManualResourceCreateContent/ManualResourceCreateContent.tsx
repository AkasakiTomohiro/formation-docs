import { zodResolver } from '@hookform/resolvers/zod';
import { useEffect, useState } from 'react';
import { useForm } from 'react-hook-form';
import { z } from 'zod';
import { useFlashbarContext } from '../../../../../../../../contexts/FlashbarContext';
import { hrefBuilder } from '../../../../../../components';
import { useWorkspaceResourceContext } from '../../../../../../contexts';
import { getAWSServiceList } from './lib/GetAWSServiceList';
import { newManualManagementResource } from './lib/NewManualManagementResource';
import { ManualResourceCreateContentPresentation } from './ManualResourceCreateContent.presentation';
import type { SelectProps } from '@cloudscape-design/components';
import type { Dispatch } from 'react';
import type { ManualResourceTabInfo } from '../../../../../../contexts';
import type { RegisteringStatus } from './ManualResourceCreateContent.presentation';

export type ManualResourceCreateContentProps = {
  /**
   * 登録中かどうかを切り替える
   */
  setRegistering: Dispatch<React.SetStateAction<boolean>>;
};

const resourceEditValidator = z.object({
  resourceId: z.string().regex(/^[A-Za-z0-9]{1,256}$/),
  description: z.string().max(256),
  serviceName: z.string().min(1).max(256),
  resourceName: z.string().min(1).max(256),
});
export type ResourceEditType = z.infer<typeof resourceEditValidator>;

export type AWSService = {
  service_name: string;
  resources: string[];
};

export const ManualResourceCreateContent = ({ setRegistering }: ManualResourceCreateContentProps): JSX.Element => {
  const [registeringStatus, setRegisteringStatus] = useState<RegisteringStatus>('SELECT_RESOURCE');
  const [services, setServices] = useState<AWSService[]>([]);
  const [selectedService, setSelectedService] = useState<SelectProps.Option | null>(null);
  const [selectedResource, setSelectedResource] = useState<SelectProps.Option | null>(null);
  const { addFlashbarItem } = useFlashbarContext();
  const { loadSideMenu, addResourceTab } = useWorkspaceResourceContext();
  const { control, setValue, handleSubmit, setError, reset } = useForm<ResourceEditType>({
    mode: 'onChange',
    resolver: zodResolver(resourceEditValidator),
    defaultValues: {
      resourceId: '',
      description: '',
      serviceName: '',
      resourceName: '',
    },
  });

  useEffect(() => {
    getAWSServiceList().then((services) => {
      setServices(
        Object.entries(services)
          .sort((a, b) => a[0].localeCompare(b[0]))
          .map((m) => ({
            service_name: m[0],
            resources: m[1].sort((x, y) => x.localeCompare(y)),
          })),
      );
    });
  }, []);

  const onChangeService: SelectProps['onChange'] = ({ detail }) => {
    if (selectedService?.value !== detail.selectedOption.value) {
      setSelectedResource(null);
    }
    setSelectedService(detail.selectedOption);
  };

  const onSave = async (data: ResourceEditType) => {
    // 選択したリソースをプロパティ空のJSONで保存する
    newManualManagementResource({
      resource_id: data.resourceId,
      description: data.description,
      service_name: data.serviceName,
      resource_name: data.resourceName,
    })
      .then(async () => {
        loadSideMenu();
        const tabId = hrefBuilder({
          type: 'manualResource',
          serviceName: data.serviceName,
          resourceType: data.resourceName,
        });
        const newTab: ManualResourceTabInfo = {
          type: 'manualResource',
          tabId: tabId,
          serviceName: data.serviceName,
          resourceName: data.resourceName,
          selectedResourceId: data.resourceId,
          viewMode: 'reason',
        };
        addResourceTab(newTab);
        setRegistering(false);
        setSelectedService(null);
        setSelectedResource(null);
        reset();
      })
      .catch((error) => {
        console.error('Error Saving resource:', error);

        if (error.value.includes('already exists')) {
          setError('resourceId', {
            type: 'already_exists',
          });
        } else {
          addFlashbarItem({
            type: 'error',
            header: '保存に失敗しました',
            content: error.value,
          });
          setRegistering(false);
          setSelectedService(null);
          setSelectedResource(null);
        }
      });
  };

  const onClickCancel = () => {
    setRegistering(false);
    setSelectedService(null);
    setSelectedResource(null);
  };

  const onClickNext = () => {
    if (selectedService !== null && selectedResource !== null) {
      setValue('serviceName', selectedService.value as string);
      setValue('resourceName', selectedResource.value as string);
      setRegisteringStatus('INPUT_NAME');
    }
  };

  return (
    <ManualResourceCreateContentPresentation
      services={services.map((item) => ({ value: item.service_name }))}
      selectedService={selectedService}
      resources={services
        .find((service) => service.service_name === selectedService?.value)
        ?.resources.map((resource) => ({ value: resource }))}
      selectedResource={selectedResource}
      registeringStatus={registeringStatus}
      onSubmit={handleSubmit(onSave)}
      formControl={control}
      onChangeService={onChangeService}
      onChangeResource={({ detail }) => setSelectedResource(detail.selectedOption)}
      onClickSelectResourceCancel={onClickCancel}
      onClickNext={onClickNext}
      onClickInputNameCancel={onClickCancel}
      onClickInputNameReturn={() => setRegisteringStatus('SELECT_RESOURCE')}
    />
  );
};
