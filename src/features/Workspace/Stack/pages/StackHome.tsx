import {
  Container,
  ContentLayout,
  Header,
} from '@cloudscape-design/components';

export type StackHomeProps = {
  stackId: string;
  stackName: string;
};

export const StackHome = (props: StackHomeProps): JSX.Element => {
  return (
    <ContentLayout header={<Header variant="h1">{props.stackName}</Header>}>
      <Container
        header={
          <Header variant="h2" description="Container description">
            Container header
          </Header>
        }
      >
        <div className="contentPlaceholder" />
        <p>{props.stackId}</p>
      </Container>
    </ContentLayout>
  );
};
