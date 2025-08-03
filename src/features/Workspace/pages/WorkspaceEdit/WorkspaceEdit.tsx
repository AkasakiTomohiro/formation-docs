import { useForm } from 'react-hook-form';
import { useNavigate, useOutletContext, useRevalidator } from 'react-router';
import { z } from 'zod';

import { zodResolver } from '@hookform/resolvers/zod';

import { useFlashbarContext } from '../../../../contexts/FlashbarContext';
import { WorkspaceEditPresentation } from './WorkspaceEdit.presentation';
import { updateWorkspace } from './lib/UpdateWorkspace';

import type { WorkspaceLayoutContext } from '../../Layout';

const workspaceEditValidator = z.object({
  name: z.string().min(1).max(256),
  description: z.string().max(256),
});

export type WorkspaceEditType = z.infer<typeof workspaceEditValidator>;

export const WorkspaceEdit = (): JSX.Element => {
  const workspace = useOutletContext<WorkspaceLayoutContext>();
  const navigate = useNavigate();
  const revalidator = useRevalidator();
  const { flashbarItems, addFlashbarItem } = useFlashbarContext();

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
      addFlashbarItem({
        type: 'error',
        header: '保存に失敗しました',
        content: typeof error === 'string' ? error : undefined,
      });
    }
  };

  return (
    <WorkspaceEditPresentation
      control={control}
      onSubmitSave={handleSubmit(onSave)}
      onClickCancel={() => navigate(`/workspaces/${workspace.id}`)}
      flashbarItems={flashbarItems}
    />
  );
};
