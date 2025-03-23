import { Route, Routes } from 'react-router';
import { Header } from './components/Header';
import { Home } from './features/Home';
import { Workspace } from './features/Workspace';

function App() {
  return (
    <Routes>
      <Route
        index
        element={
          <Header>
            <Home />
          </Header>
        }
      />
      <Route
        path="/workspaces/:workspaceId"
        element={
          <Header>
            <Workspace />
          </Header>
        }
      />
    </Routes>
  );
}

export default App;
