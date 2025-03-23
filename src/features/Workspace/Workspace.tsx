import { useParams } from 'react-router';

export const Workspace = (): JSX.Element => {
  const { workspaceId } = useParams();
  return <div>Workspace({workspaceId})</div>;
};
