import TopNavigation from '@cloudscape-design/components/top-navigation';
import { invoke } from '@tauri-apps/api/core';
import { Window } from '@tauri-apps/api/window';

import type { HeaderProps } from './types';
export const Header = (props: HeaderProps): JSX.Element => {
  return (
    <>
      <TopNavigation
        identity={{
          href: '/',
          title: 'FormationDocs',
          onFollow: (event) => {
            event.preventDefault();
            Window.getByLabel('main').then((mainWindow) => {
              if (mainWindow) {
                mainWindow.setFocus();
              } else {
                invoke('open_workspace_command', {
                  id: 'main',
                });
              }
            });
          },
        }}
      />
      {props.children}
    </>
  );
};
