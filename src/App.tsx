import { createBrowserRouter, RouterProvider } from "react-router";

import { Header } from "./components/Header";
import { AppSetup } from "./features/AppSetup";
import {
  StackEdit,
  StackHome,
  StackLayout,
  stackLoader,
  WorkspaceEdit,
  WorkspaceHome,
  WorkspaceLayout,
  workspaceLoader,
} from "./features/Workspace";
import { Workspaces } from "./features/Workspaces";

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
    path: "workspaces",
    element: (
      <Header>
        <Workspaces />
      </Header>
    ),
  },
  {
    path: "workspaces/:workspaceId",
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
        path: "edit",
        element: <WorkspaceEdit />,
      },
      {
        path: "stacks/:stackId",
        element: <StackLayout />,
        loader: stackLoader,
        children: [
          {
            index: true,
            element: <StackHome />,
          },
          {
            path: "edit",
            element: <StackEdit />,
          },
        ],
      },
    ],
  },
]);

function App() {
  return <RouterProvider router={router} />;
}

export default App;
