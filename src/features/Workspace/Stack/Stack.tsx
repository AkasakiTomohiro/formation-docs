import { useNavigate, useOutletContext, useParams } from 'react-router';

import {
  AppLayout,
  BreadcrumbGroup,
  Container,
  ContentLayout,
  Header,
  SideNavigation,
} from '@cloudscape-design/components';

import type { WorkspaceLayoutLoaderData } from '../Loader';

export const Stack = (): JSX.Element => {
  const workspace = useOutletContext<WorkspaceLayoutLoaderData>();
  const navigate = useNavigate();
  const params = useParams();
  const stackId = params.stackId as string;
  return (
    <AppLayout
      toolsHide
      breadcrumbs={
        <BreadcrumbGroup
          items={[
            { text: workspace.name, href: '#' },
            { text: stackId, href: '#' },
          ]}
          onClick={(e) => {
            if (e.detail.text === workspace.name) {
              navigate(`/workspaces/${workspace.id}`);
            }
          }}
        />
      }
      navigationOpen={true}
      navigation={
        <SideNavigation
          header={{
            href: '#',
            text: workspace.name,
          }}
          items={[{ type: 'link', text: 'Page #1', href: '#' }]}
        />
      }
      // notifications={
      //   <Flashbar
      //     items={[
      //       {
      //         type: 'info',
      //         dismissible: true,
      //         content: 'This is an info flash message.',
      //         id: 'message_1',
      //       },
      //     ]}
      //   />
      // }
      content={
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
      }
    />
  );
};
