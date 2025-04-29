import type { LoaderFunctionArgs } from 'react-router';
import { loadStack } from '../../../invoke/Stack';
import { loadWorkspace } from '../../../invoke/Workspace';

import type { StackInfo } from '../pages';

export type StackLayoutLoaderData = {
  stack: StackInfo;
};

export const stackLoader = async ({
  params,
}: LoaderFunctionArgs): Promise<StackLayoutLoaderData> => {
  const workspace = await loadWorkspace(params.workspaceId as string);
  const stack = await loadStack(workspace.directory, params.stackId as string);
  return {
    stack,
  };
};
