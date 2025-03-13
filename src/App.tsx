import { Route, Routes } from 'react-router';
import { Header } from './components/Header';
import { Home } from './features/Home';

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
    </Routes>
  );
}

export default App;
