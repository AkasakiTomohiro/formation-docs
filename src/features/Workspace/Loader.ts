import { loadWorkspace } from '../../invoke/Workspace';

import type { LoaderFunctionArgs } from 'react-router';
import type { WorkspaceExpand } from '../../hooks/useWorkspaces';

export type WorkspaceLayoutLoaderData = WorkspaceExpand;

export const workspaceLoader = async ({ params }: LoaderFunctionArgs): Promise<WorkspaceLayoutLoaderData> => {
  return loadWorkspace(params.workspaceId as string);
};
