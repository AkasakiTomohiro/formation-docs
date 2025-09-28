import { LogicalIdTable } from './components/LogicalIdTable';
import { ResourcePropertyTable } from './components/ResourcePropertyTable';
import type { ResourceTabAttr } from '../../../../contexts';
export type ResourceTabProps = ResourceTabAttr;

export type BuildResourceTabNameProps = {
  stackName: string;
  serviceName: string;
  resourceName: string;
};

export function buildResourceTabName({ stackName, serviceName, resourceName }: BuildResourceTabNameProps): string {
  return `${stackName} - ${serviceName}::${resourceName}`;
}

export const ResourceTab = (props: ResourceTabProps): JSX.Element => {
  return (
    <div
      style={{
        width: '100%',
        height: '100%',
        display: 'flex',
      }}
    >
      <LogicalIdTable
        tabId={props.tabId}
        stackId={props.stackId}
        serviceName={props.serviceName}
        resourceName={props.resourceName}
        selectedLogicalId={props.selectedLogicalId}
      />
      {props.selectedLogicalId !== undefined && (
        <ResourcePropertyTable
          stackId={props.stackId}
          stackName={props.stackName}
          serviceName={props.serviceName}
          resourceName={props.resourceName}
          selectedLogicalId={props.selectedLogicalId}
        />
      )}
    </div>
  );
};
