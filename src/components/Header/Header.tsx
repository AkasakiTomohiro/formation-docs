import TopNavigation from '@cloudscape-design/components/top-navigation';

import type { HeaderProps } from './types';

export const Header = (props: HeaderProps): JSX.Element => {
  return (
    <>
      <TopNavigation
        identity={{
          href: '/',
          title: 'FormationDocs',
        }}
      />
      {props.children}
    </>
  );
};
