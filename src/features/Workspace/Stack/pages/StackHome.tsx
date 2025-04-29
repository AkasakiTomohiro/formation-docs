import { useParams } from "react-router";

import {
  Container,
  ContentLayout,
  Header,
} from "@cloudscape-design/components";

export const StackHome = (): JSX.Element => {
  const params = useParams();
  const stackId = params.stackId as string;
  return (
    <ContentLayout header={<Header variant="h1">{stackId}</Header>}>
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
