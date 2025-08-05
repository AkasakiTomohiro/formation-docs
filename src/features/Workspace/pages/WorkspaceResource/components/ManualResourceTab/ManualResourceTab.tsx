import { ResourceIdTable } from './components';
import { EditorContentLayout } from './components/EditorContentLayout';

import type { ManualResourceTabAttr } from '../../../../contexts';

export type ManualResourceTabProps = ManualResourceTabAttr;

export type BuildManualResourceTabNameProps = {
  serviceName: string;
  resourceName: string;
};
export function buildManualResourceTabName({ serviceName, resourceName }: BuildManualResourceTabNameProps): string {
  return `手動管理リソース - ${serviceName}::${resourceName}`;
}

export const ManualResourceTab = (props: ManualResourceTabProps): JSX.Element => {
  return (
    <div
      style={{
        width: '100%',
        height: '100%',
        display: 'flex',
      }}
    >
      <ResourceIdTable
        tabId={props.tabId}
        selectedResourceId={props.selectedResourceId}
        serviceName={props.serviceName}
        resourceName={props.resourceName}
      />
      {props.selectedResourceId !== undefined && (
        <div style={{ paddingLeft: '16px' }}>
          <EditorContentLayout
            selectedResourceId={props.selectedResourceId}
            serviceName={props.serviceName}
            resourceName={props.resourceName}
          />
        </div>
      )}
    </div>
  );
};
