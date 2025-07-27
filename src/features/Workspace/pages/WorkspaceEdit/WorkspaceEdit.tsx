import { useState } from 'react';
import { useForm } from 'react-hook-form';
import { useNavigate, useOutletContext, useRevalidator } from 'react-router';
import { v4 as uuidV4 } from 'uuid';
import { z } from 'zod';

import { zodResolver } from '@hookform/resolvers/zod';

import { WorkspaceEditPresentation } from './WorkspaceEdit.presentation';
import { updateWorkspace } from './lib/UpdateWorkspace';

import type { WorkspaceLayoutContext } from '../../Layout';

import type { FlashbarProps } from '@cloudscape-design/components';

const workspaceEditValidator = z.object({
  name: z.string().min(1).max(256),
  description: z.string().max(256),
});

export type WorkspaceEditType = z.infer<typeof workspaceEditValidator>;

export const WorkspaceEdit = (): JSX.Element => {
  const workspace = useOutletContext<WorkspaceLayoutContext>();
  const navigate = useNavigate();
  const revalidator = useRevalidator();
  const [flashbarItems, setFlashbarItems] = useState<FlashbarProps.MessageDefinition[]>([]);
  const { control, handleSubmit } = useForm<WorkspaceEditType>({
    mode: 'onChange',
    resolver: zodResolver(workspaceEditValidator),
    defaultValues: {
      name: workspace.name,
      description: workspace.description,
    },
  });

  const onSave = async (newWorkspace: WorkspaceEditType) => {
    try {
      await updateWorkspace({
        name: newWorkspace.name,
        description: newWorkspace.description,
      });

      // ローダーの再読み込み
      await revalidator.revalidate();

      navigate(`/workspaces/${workspace.id}`);
    } catch (error) {
      const id = uuidV4();
      setFlashbarItems([
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
    <WorkspaceEditPresentation
      control={control}
      onSubmitSave={handleSubmit(onSave)}
      onClickCancel={() => navigate(`/workspaces/${workspace.id}`)}
    />
  );
};
