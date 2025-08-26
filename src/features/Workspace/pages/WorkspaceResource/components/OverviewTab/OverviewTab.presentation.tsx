import { StackDisplayContent } from './components/StackDisplayContent/StackDisplayContent';
import { StackEditContent } from './components/StackEditContent';

import type { FlashbarProps } from '@cloudscape-design/components';
import type { OverviewTabAttr } from '../../../../contexts';

export type OverviewTabPresentationProps = OverviewTabAttr & {
  /**
   * フラッシュバーのアイテム
   */
  flashbarItems: FlashbarProps.MessageDefinition[];

  /**
   * フラッシュバーにメッセージを追加する関数
   */
  setFlashbarItems: React.Dispatch<React.SetStateAction<FlashbarProps.MessageDefinition[]>>;
};
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
  flashbarItems,
  setFlashbarItems,
  editingValues,
}: OverviewTabPresentationProps): JSX.Element => {
  if (editingValues) {
    return (
      <StackEditContent
        tabId={tabId}
        stackId={stackId}
        stackName={editingValues.stackName}
        stackDescription={editingValues.stackDescription}
        flashbarItems={flashbarItems}
        setFlashbarItems={setFlashbarItems}
      />
    );
  }
  return (
    <StackDisplayContent
      tabId={tabId}
      stackId={stackId}
      stackName={stackName}
      stackDescription={description}
      flashbarItems={flashbarItems}
    />
  );
};
