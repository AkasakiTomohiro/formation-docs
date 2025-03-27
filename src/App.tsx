import { RouterProvider, createBrowserRouter } from 'react-router';
import { Header } from './components/Header';
import { Home } from './features/Home';
import {
  WorkspaceHome,
  WorkspaceLayout,
  workspaceLoader,
} from './features/Workspace';
import { WorkspaceEdit } from './features/Workspace/components/WorkspaceEdit';

const router = createBrowserRouter([
  {
    index: true,
    element: (
      <Header>
        <Home />
      </Header>
    ),
  },
  {
    path: '/workspaces/:workspaceId',
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
    ],
  },
]);

function App() {
  return <RouterProvider router={router} />;
}

export default App;
