import { setupApp } from '../../invoke/AppConfig';

export const workspacesLoader = async (): Promise<void> => {
  await setupApp();
};
