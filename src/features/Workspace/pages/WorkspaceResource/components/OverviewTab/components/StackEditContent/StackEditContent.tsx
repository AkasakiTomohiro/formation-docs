import { type Dispatch, useEffect } from 'react';
import { useForm } from 'react-hook-form';
import { v4 as uuidV4 } from 'uuid';

import { zodResolver } from '@hookform/resolvers/zod';

import { useWorkspaceResourceContext } from '../../../../../../contexts';
import { StackEditContentPresentation, StackEditValidator } from './StackEditContent.presentation';
import { updateStackDetail } from './lib/UpdateStackDetail';

import type { FlashbarProps } from '@cloudscape-design/components';
import type { OverviewTabInfo } from '../../../../../../contexts';
import type { StackEditType } from './StackEditContent.presentation';

export type StackEditContentProps = {
  /**
   * タブID
   */
  tabId: string;

  /**
   * スタックID
   */
  stackId: string;

  /**
   * スタック名
   */
  stackName: string;

  /**
   * スタックの説明
   */
  stackDescription: string;

  /**
   * フラッシュバーに表示するメッセージ
   */
  flashbarItems: FlashbarProps.MessageDefinition[];

  /**
   * フラッシュバーにメッセージを追加する関数
   */
  setFlashbarItems: Dispatch<React.SetStateAction<FlashbarProps.MessageDefinition[]>>;
};

export const StackEditContent = ({
  stackName,
  stackDescription,
  tabId,
  stackId,
  flashbarItems,
  setFlashbarItems,
}: StackEditContentProps): JSX.Element => {
  const { modifyResourceTab } = useWorkspaceResourceContext();
  const { control, handleSubmit, watch } = useForm<StackEditType>({
    mode: 'onChange',
    resolver: zodResolver(StackEditValidator),
    defaultValues: {
      name: stackName,
      description: stackDescription,
    },
  });
  const [description, name] = watch(['description', 'name']);

  // スタック名と説明が変更されるたびにコンテキストにデータを保持
  useEffect(() => {
    console.log('StackEditContent useEffect in stack edit content', { description, name });
    modifyResourceTab(tabId, (originTab: OverviewTabInfo) => {
      return {
        ...originTab,
        editingValues: {
          stackName: name,
          stackDescription: description,
        },
      };
    });
  }, [description, name, modifyResourceTab, tabId]);

  const onSave = async (stackDetailProps: StackEditType) => {
    try {
      await updateStackDetail({
        stack_id: stackId,
        name: stackDetailProps.name,
        description: stackDetailProps.description,
      });

      // 保存したスタックの情報を更新
      modifyResourceTab(tabId, (originTab: OverviewTabInfo) => {
        return {
          ...originTab,
          stackName: stackDetailProps.name,
          description: stackDetailProps.description,
          editingValues: undefined,
        };
      });
    } catch (error) {
      const id = uuidV4();
      setFlashbarItems(() => [
        ...flashbarItems,
        {
          type: 'error',
          header: '保存に失敗しました',
          content: typeof error === 'string' ? error : undefined,
          dismissible: true,
          dismissLabel: 'close',
          id: id,
          onDismiss: () => {
            setFlashbarItems((items) => items.filter((e) => e.id !== id));
          },
        },
      ]);
    }
  };
  return (
    <StackEditContentPresentation
      control={control}
      onSubmitSave={handleSubmit(onSave)}
      flashbarItems={flashbarItems}
      onClickCancel={() => {
        modifyResourceTab(tabId, (originTab: OverviewTabInfo) => {
          return {
            ...originTab,
            editingValues: undefined,
          };
        });
      }}
    />
  );
};
