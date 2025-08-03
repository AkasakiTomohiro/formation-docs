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
  isEdit,
  tabId,
  stackId,
  stackName,
  description,
}: OverviewTabPresentationProps): JSX.Element => {
  if (isEdit) {
    return <StackEditContent tabId={tabId} stackId={stackId} stackName={stackName} stackDescription={description} />;
  }
  return <StackDisplayContent tabId={tabId} stackId={stackId} stackName={stackName} stackDescription={description} />;
};
