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
  isEdit,
  tabId,
  stackId,
  stackName,
  description,
  flashbarItems,
  setFlashbarItems,
}: OverviewTabPresentationProps): JSX.Element => {
  if (isEdit) {
    return (
      <StackEditContent
        tabId={tabId}
        stackId={stackId}
        stackName={stackName}
        stackDescription={description}
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
