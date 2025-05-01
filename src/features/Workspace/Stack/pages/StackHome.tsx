import { useOutletContext } from 'react-router';

import {
  Container,
  ContentLayout,
  Header,
} from '@cloudscape-design/components';

import type { StackLayoutContext } from '../Layout';

export const StackHome = (): JSX.Element => {
  const context = useOutletContext<StackLayoutContext>();
  return (
    <ContentLayout header={<Header variant="h1">{context.stack.name}</Header>}>
      <Container
        header={
          <Header variant="h2" description="Container description">
            Container header
          </Header>
        }
      >
        <div className="contentPlaceholder" />
      </Container>
    </ContentLayout>
  );
};
