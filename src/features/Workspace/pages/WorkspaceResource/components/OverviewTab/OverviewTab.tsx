import { useEffect } from 'react';
import { useWorkspaceResourceContext } from '../../../../contexts';
import { loadStack } from './lib/LoadStack';
import { OverviewTabPresentation } from './OverviewTab.presentation';
import type { OverviewTabAttr, OverviewTabInfo } from '../../../../contexts';

export type OverviewTabProps = OverviewTabAttr;

export type BuildOverviewTabNameProps = {
  stackName: string;
};
export function buildOverviewTabName({ stackName }: BuildOverviewTabNameProps): string {
  return stackName;
}

export const OverviewTab = (props: OverviewTabProps): JSX.Element => {
  const { modifyResourceTab } = useWorkspaceResourceContext();

  // biome-ignore lint/correctness/useExhaustiveDependencies: false positive
  useEffect(() => {
    loadStack(props.stackId).then((stackDetail) => {
      const stackName = stackDetail.name;
      const description = stackDetail.description_from_meta;

      // 取得したスタック情報を更新
      modifyResourceTab(props.tabId, (originTab: OverviewTabInfo) => {
        return {
          ...originTab,
          stackName: stackName,
          description: description,
        };
      });
    });
  }, []);

  return (
    <OverviewTabPresentation
      isEdit={props.isEdit}
      tabId={props.tabId}
      stackId={props.stackId}
      stackName={props.stackName}
      description={props.description}
    />
  );
};
