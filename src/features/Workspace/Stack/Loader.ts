import { loadStack } from '../../../invoke/Stack';

import type { LoaderFunctionArgs } from 'react-router';
import type { StackInfo } from '../../../invoke/Stack';

export type StackLayoutLoaderData = {
  stack: StackInfo;
};

export const stackLoader = async ({
  params,
}: LoaderFunctionArgs): Promise<StackLayoutLoaderData> => {
  const stack = await loadStack(params.stackId as string);
  return {
    stack,
  };
};
