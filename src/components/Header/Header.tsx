import { Flashbar } from '@cloudscape-design/components';
import TopNavigation from '@cloudscape-design/components/top-navigation';
import { invoke } from '@tauri-apps/api/core';
import { Window } from '@tauri-apps/api/window';
import { useFlashbarContext } from '../../contexts/FlashbarContext';
import type { HeaderProps } from './types';

export const Header = (props: HeaderProps): JSX.Element => {
  const { flashbarItems } = useFlashbarContext();

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
        utilities={[
          {
            type: 'button',
            iconName: 'settings',
            title: 'Settings',
            onClick: () => {
              Window.getByLabel('settings').then((settingsWindow) => {
                if (settingsWindow) {
                  settingsWindow.setFocus();
                } else {
                  invoke('open_settings_command');
                }
              });
            },
          },
        ]}
      />
      <div style={flashbarItems.length > 0 ? { marginTop: 16, marginLeft: 16, marginRight: 16 } : {}}>
        <Flashbar items={flashbarItems} />
      </div>
      {props.children}
    </>
  );
};
