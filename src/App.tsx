import { RouterProvider, createBrowserRouter } from 'react-router';

import { Header } from './components/Header';
import { FlashbarProvider } from './contexts/FlashbarContext';
import { AppHome } from './features/AppHome';
import { AppSetup } from './features/AppSetup';
import {
  WorkspaceEdit,
  WorkspaceHome,
  WorkspaceLayout,
  WorkspaceResource,
  workspaceLoader,
} from './features/Workspace';
import { WorkspaceResourceProvider } from './features/Workspace/contexts/WorkspaceResourceContext';

const router = createBrowserRouter([
  {
    index: true,
    element: (
      <Header>
        <AppSetup />
      </Header>
    ),
  },
  {
    path: 'workspaces',
    element: (
      <Header>
        <AppHome />
      </Header>
    ),
  },
  {
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
    ],
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
