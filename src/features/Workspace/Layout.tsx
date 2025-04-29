import { Outlet, useLoaderData } from "react-router";

import type { WorkspaceLayoutLoaderData } from "./Loader";

export type WorkspaceLayoutContext = WorkspaceLayoutLoaderData;

export const WorkspaceLayout = (): JSX.Element => {
  const result = useLoaderData<WorkspaceLayoutLoaderData>();
  return <Outlet context={result} />;
};
