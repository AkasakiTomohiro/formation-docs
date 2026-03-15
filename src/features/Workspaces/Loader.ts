import { setupApp } from '../../invoke/AppConfig';
import type { SetupAppResult } from '../../invoke/AppConfig';

export type WorkspacesLoaderData = SetupAppResult;

export const workspacesLoader = async (): Promise<WorkspacesLoaderData> => {
  return setupApp();
};
