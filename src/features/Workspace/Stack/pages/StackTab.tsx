import {
  Container,
  ContentLayout,
  Header,
} from '@cloudscape-design/components';

export type StackTabProps = {
  stackId: string;
  stackName: string;
};

export const StackTab = (props: StackTabProps): JSX.Element => {
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
