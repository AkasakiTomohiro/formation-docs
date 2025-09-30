import { StackDisplayContent } from './components/StackDisplayContent/StackDisplayContent';
import { StackEditContent } from './components/StackEditContent';
import type { OverviewTabAttr } from '../../../../contexts';

export type OverviewTabPresentationProps = OverviewTabAttr;

export type BuildOverviewTabNameProps = {
  stackName: string;
};
export function buildOverviewTabName({ stackName }: BuildOverviewTabNameProps): string {
  return stackName;
}

export const OverviewTabPresentation = ({
  tabId,
  stackId,
  stackName,
  description,
  editingValues,
}: OverviewTabPresentationProps): JSX.Element => {
  if (editingValues) {
    return (
      <StackEditContent
        tabId={tabId}
        stackId={stackId}
        stackName={editingValues.stackName}
        stackDescription={editingValues.stackDescription}
      />
    );
  }
  return <StackDisplayContent tabId={tabId} stackId={stackId} stackName={stackName} stackDescription={description} />;
};
