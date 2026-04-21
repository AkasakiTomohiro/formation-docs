import { createBrowserRouter, RouterProvider } from 'react-router';
import { Header } from './components/Header';
import { FlashbarProvider } from './contexts/FlashbarContext';
import { ErrorScreen } from './features/ErrorScreen';
import {
  ResourceDescriptionSettings,
  WorkspaceEdit,
  WorkspaceHome,
  WorkspaceLayout,
  WorkspaceResource,
  workspaceLoader,
} from './features/Workspace';
import { WorkspaceResourceProvider } from './features/Workspace/contexts/WorkspaceResourceContext';
import { Workspaces } from './features/Workspaces';
import { workspacesLoader } from './features/Workspaces/Loader';

const router = createBrowserRouter([
  {
    id: 'workspaces',
    index: true,
    loader: workspacesLoader,
    element: (
      <Header>
        <Workspaces />
      </Header>
    ),
  },
  {
    id: 'workspace',
    path: 'workspaces/:workspaceId',
    element: (
      <Header>
        <WorkspaceResourceProvider>
          <WorkspaceLayout />
        </WorkspaceResourceProvider>
      </Header>
    ),
    loader: workspaceLoader,
    children: [
      {
        index: true,
        element: <WorkspaceHome />,
      },
      {
        path: 'edit',
        element: <WorkspaceEdit />,
      },
      {
        path: 'resources',
        element: <WorkspaceResource />,
      },
      {
        path: 'resourceDescriptionSettings',
        element: <ResourceDescriptionSettings />,
      },
    ],
  },
  {
    id: 'errorScreen',
    path: 'errorScreen',
    element: (
      <Header>
        <ErrorScreen />
      </Header>
    ),
  },
]);

function App() {
  return (
    <FlashbarProvider>
      <RouterProvider router={router} />
    </FlashbarProvider>
  );
}

export default App;
