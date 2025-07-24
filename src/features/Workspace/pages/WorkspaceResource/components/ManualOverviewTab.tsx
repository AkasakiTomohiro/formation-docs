import { useEffect, useState } from 'react';
import { Controller, useForm } from 'react-hook-form';
import { useOutletContext } from 'react-router';
import { v4 as uuidV4 } from 'uuid';
import { z } from 'zod';

import {
  Box,
  Button,
  Container,
  ContentLayout,
  FormField,
  Header,
  Input,
  Select,
  SpaceBetween,
  TextContent,
  Textarea,
} from '@cloudscape-design/components';
import { zodResolver } from '@hookform/resolvers/zod';

import { getAWSServiceList } from '../../../../../invoke/CloudFormationSchema';
import { newManualManagementResource } from '../../../../../invoke/ManualManagementResource';
import { hrefBuilder } from '../../../components/WorkspaceSideMenu';
import { useWorkspaceResourceContext } from '../../../contexts';

import type { WorkspaceLayoutContext } from '../../../Layout';
import type { ManualResourceTabInfo } from '../../../contexts';

import type { SelectProps } from '@cloudscape-design/components';

const resourceEditValidator = z.object({
  resourceId: z.string().regex(/^[A-Za-z0-9]{1,256}$/),
  description: z.string().max(256),
  serviceName: z.string().min(1).max(256),
  resourceName: z.string().min(1).max(256),
});

type WorkspaceEditType = z.infer<typeof resourceEditValidator>;
export type AWSService = {
  service_name: string;
  resources: string[];
};

export function buildManualOverviewTabName(): string {
  return '手動管理リソース';
}

export const ManualOverviewTab = (): JSX.Element => {
  const [services, setServices] = useState<AWSService[]>([]);
  const [registering, setRegistering] = useState<'SELECT_RESOURCE' | 'INPUT_NAME' | null>(null);
  const [selectedService, setSelectedService] = useState<SelectProps.Option | null>(null);
  const [selectedResource, setSelectedResource] = useState<SelectProps.Option | null>(null);
  const { control, setValue, handleSubmit, setError, reset } = useForm<WorkspaceEditType>({
    mode: 'onChange',
    resolver: zodResolver(resourceEditValidator),
    defaultValues: {
      resourceId: '',
      description: '',
      serviceName: '',
      resourceName: '',
    },
  });
  const { flashbarItems, setFlashbarItems } = useOutletContext<WorkspaceLayoutContext>();
  const { loadSideMenu, addResourceTab } = useWorkspaceResourceContext();

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

  const onSave = async (data: WorkspaceEditType) => {
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
        };
        addResourceTab(newTab);
        setRegistering(null);
        setSelectedService(null);
        setSelectedResource(null);
        reset();
      })
      .catch((error) => {
        const id = uuidV4();
        console.error('Error Saving resource:', error);

        if (error.value.includes('already exists')) {
          setError('resourceId', {
            type: 'already_exists',
          });
        } else {
          setFlashbarItems([
            ...flashbarItems,
            {
              type: 'error',
              header: '保存に失敗しました',
              content: error.value,
              dismissible: true,
              dismissLabel: 'close',
              id: id,
              onDismiss: () => {
                setFlashbarItems((items) => items.filter((e) => e.id !== id));
              },
            },
          ]);
          setRegistering(null);
          setSelectedService(null);
          setSelectedResource(null);
        }
      });
  };

  return (
    <ContentLayout header={<Header variant="h1">手動管理リソース</Header>}>
      {/* リソース選択画面 */}
      {registering === 'SELECT_RESOURCE' && (
        <SpaceBetween direction="vertical" size="m">
          <Container header={<Header variant="h2">リソースを選択</Header>}>
            <SpaceBetween direction="vertical" size="s">
              <Select
                placeholder="サービス名"
                selectedOption={selectedService}
                onChange={({ detail }) => {
                  if (selectedService?.value !== detail.selectedOption.value) {
                    setSelectedResource(null);
                  }
                  setSelectedService(detail.selectedOption);
                }}
                options={services.map((item) => ({
                  value: item.service_name,
                }))}
                filteringType="auto"
              />
              <Select
                placeholder="リソース名"
                disabled={!selectedService}
                selectedOption={selectedResource}
                onChange={({ detail }) => setSelectedResource(detail.selectedOption)}
                options={services
                  .find((service) => service.service_name === selectedService?.value)
                  ?.resources.map((resource) => ({ value: resource }))}
                filteringType="auto"
              />
            </SpaceBetween>
          </Container>
          <Box float="right">
            <SpaceBetween direction="horizontal" size="xs">
              <Button
                onClick={() => {
                  setRegistering(null);
                  setSelectedService(null);
                  setSelectedResource(null);
                }}
              >
                キャンセル
              </Button>
              <Button
                variant="primary"
                onClick={() => {
                  if (selectedService !== null && selectedResource !== null) {
                    setValue('serviceName', selectedService.value as string);
                    setValue('resourceName', selectedResource.value as string);
                    setRegistering('INPUT_NAME');
                  }
                }}
              >
                次へ
              </Button>
            </SpaceBetween>
          </Box>
        </SpaceBetween>
      )}
      {/* リソースID設定画面 */}
      {registering === 'INPUT_NAME' && (
        <form onSubmit={handleSubmit(onSave)}>
          <SpaceBetween direction="vertical" size="m">
            <Container header={<Header variant="h2">リソースの詳細</Header>}>
              <SpaceBetween direction="vertical" size="s">
                <Controller
                  name="resourceId"
                  control={control}
                  render={({ field, fieldState: { invalid, error } }) => {
                    return (
                      <FormField
                        label="Resource ID"
                        errorText={
                          invalid
                            ? error?.type === 'already_exists'
                              ? 'このResource IDはすでに存在します'
                              : '1文字以上256文字以下の半角英数字で入力してください'
                            : undefined
                        }
                      >
                        <Input {...field} onChange={(event) => field.onChange(event.detail.value)} invalid={invalid} />
                      </FormField>
                    );
                  }}
                />
                <Controller
                  name="description"
                  control={control}
                  render={({ field, fieldState: { invalid } }) => {
                    return (
                      <FormField label="Description" errorText={invalid && '256文字以下で入力してください'}>
                        <Textarea
                          {...field}
                          onChange={(event) => field.onChange(event.detail.value)}
                          invalid={invalid}
                        />
                      </FormField>
                    );
                  }}
                />
              </SpaceBetween>
            </Container>
            <Box float="right">
              <SpaceBetween direction="horizontal" size="xs">
                <Button
                  onClick={() => {
                    setRegistering(null);
                    setSelectedService(null);
                    setSelectedResource(null);
                  }}
                >
                  キャンセル
                </Button>
                <Button onClick={() => setRegistering('SELECT_RESOURCE')}>戻る</Button>
                <Button variant="primary" formAction="submit">
                  保存
                </Button>
              </SpaceBetween>
            </Box>
          </SpaceBetween>
        </form>
      )}
      {/* Overview */}
      {registering === null && (
        <SpaceBetween direction="vertical" size="s">
          <TextContent>
            <p>CloudFormationで管理していないAWSリソースの設定を管理する機能です。</p>
          </TextContent>
          <Box float="right">
            <Button variant="primary" onClick={() => setRegistering('SELECT_RESOURCE')}>
              新規リソース作成
            </Button>
          </Box>
        </SpaceBetween>
      )}
    </ContentLayout>
  );
};
