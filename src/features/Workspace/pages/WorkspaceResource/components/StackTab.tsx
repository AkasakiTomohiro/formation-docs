import {
  Button,
  Container,
  ContentLayout,
  Header,
  SpaceBetween,
} from '@cloudscape-design/components';

export type StackTabProps = {
  stackId: string;
  stackName: string;
};

export const StackTab = (props: StackTabProps): JSX.Element => {
  return (
    <ContentLayout
      header={
        <SpaceBetween size="m">
          <Header
            variant="h1"
            actions={
              <Button variant="normal" onClick={() => {}}>
                編集
              </Button>
            }
          >
            {props.stackName}
          </Header>
          {/* <Flashbar items={flashbarItems} /> */}
        </SpaceBetween>
      }
    >
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
