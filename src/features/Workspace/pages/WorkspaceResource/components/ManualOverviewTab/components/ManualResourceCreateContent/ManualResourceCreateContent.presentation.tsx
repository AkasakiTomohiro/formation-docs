import {
  Box,
  Button,
  Container,
  FormField,
  Header,
  Input,
  Select,
  SpaceBetween,
  Textarea,
} from '@cloudscape-design/components';
import { Controller } from 'react-hook-form';
import type { ButtonProps, SelectProps } from '@cloudscape-design/components';
import type { FormEventHandler } from 'react';
import type { Control } from 'react-hook-form';
import type { ResourceEditType } from './ManualResourceCreateContent';

export type RegisteringStatus = 'SELECT_RESOURCE' | 'INPUT_NAME';

export type ManualResourceCreateContentPresentationProps = {
  /**
   * 登録画面の状態
   */
  registeringStatus: RegisteringStatus;

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
   * リソース選択画面のキャンセルボタン押下時のイベントハンドラ
   */
  onClickSelectResourceCancel: ButtonProps['onClick'];

  /**
   * リソース選択画面の次へボタン押下時のイベントハンドラ
   */
  onClickNext: ButtonProps['onClick'];

  /**
   * React Hook FormのControlオブジェクト
   */
  formControl: Control<ResourceEditType>;

  /**
   * 保存ボタン押下時のイベントハンドラ
   */
  onSubmit: FormEventHandler<HTMLFormElement>;

  /**
   * リソースID設定画面のキャンセルボタン押下時のイベントハンドラ
   */
  onClickInputNameCancel: ButtonProps['onClick'];

  /**
   * リソースID設定画面の戻るボタン押下時のイベントハンドラ
   */
  onClickInputNameReturn: ButtonProps['onClick'];
};

export const ManualResourceCreateContentPresentation = ({
  registeringStatus,
  services,
  selectedService,
  onChangeService,
  resources,
  selectedResource,
  onChangeResource,
  onClickSelectResourceCancel,
  onClickNext,
  formControl,
  onClickInputNameCancel,
  onClickInputNameReturn,
  onSubmit,
}: ManualResourceCreateContentPresentationProps): JSX.Element => {
  if (registeringStatus === 'SELECT_RESOURCE') {
    return (
      <SpaceBetween direction="vertical" size="m">
        <Container header={<Header variant="h2">リソースを選択</Header>}>
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
          </SpaceBetween>
        </Container>
        <Box float="right">
          <SpaceBetween direction="horizontal" size="xs">
            <Button onClick={onClickSelectResourceCancel}>キャンセル</Button>
            <Button variant="primary" onClick={onClickNext}>
              次へ
            </Button>
          </SpaceBetween>
        </Box>
      </SpaceBetween>
    );
  }

  return (
    <form onSubmit={onSubmit}>
      <SpaceBetween direction="vertical" size="m">
        <Container header={<Header variant="h2">リソースの詳細</Header>}>
          <SpaceBetween direction="vertical" size="s">
            <Controller
              name="resourceId"
              control={formControl}
              render={({ field, fieldState: { invalid, error } }) => {
                return (
                  <FormField
                    label="リソースID"
                    errorText={
                      invalid
                        ? error?.type === 'already_exists'
                          ? 'このリソースIDはすでに存在します'
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
              control={formControl}
              render={({ field, fieldState: { invalid } }) => {
                return (
                  <FormField label="説明" errorText={invalid && '256文字以下で入力してください'}>
                    <Textarea {...field} onChange={(event) => field.onChange(event.detail.value)} invalid={invalid} />
                  </FormField>
                );
              }}
            />
          </SpaceBetween>
        </Container>
        <Box float="right">
          <SpaceBetween direction="horizontal" size="xs">
            <Button onClick={onClickInputNameCancel}>キャンセル</Button>
            <Button onClick={onClickInputNameReturn}>戻る</Button>
            <Button variant="primary" formAction="submit">
              保存
            </Button>
          </SpaceBetween>
        </Box>
      </SpaceBetween>
    </form>
  );
};
