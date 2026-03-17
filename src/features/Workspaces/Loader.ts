import { redirect } from 'react-router';
import { setupApp } from '../../invoke/AppConfig';

export const workspacesLoader = async (): Promise<void> => {
  try {
    await setupApp();
  } catch {
    throw redirect('/errorScreen');
  }
};
