import { RouterProvider, createBrowserRouter } from 'react-router';

import { Header } from './components/Header';
import { AppHome } from './features/AppHome';
import { AppSetup } from './features/AppSetup';
import {
  WorkspaceEdit,
  WorkspaceHome,
  WorkspaceLayout,
  WorkspaceResource,
  workspaceLoader,
} from './features/Workspace';

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
        <WorkspaceLayout />
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
  return <RouterProvider router={router} />;
}

export default App;
