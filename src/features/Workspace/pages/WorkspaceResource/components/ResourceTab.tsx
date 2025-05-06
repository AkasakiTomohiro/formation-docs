type ResourceTabProps = {
  stackId: string;
  stackName: string;
  serviceName: string;
  resourceName: string;
};

export const ResourceTab = (props: ResourceTabProps): JSX.Element => {
  return <div>{JSON.stringify(props)}</div>;
};
