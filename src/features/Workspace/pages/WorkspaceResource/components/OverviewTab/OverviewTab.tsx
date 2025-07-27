import { useEffect, useState } from 'react';
import { z } from 'zod';

import { loadStack } from '../../../../../../invoke/Stack';
import { useWorkspaceResourceContext } from '../../../../contexts';
import { StackDisplayContent } from './components/StackDisplayContent/StackDisplayContent';
import { StackEditContent } from './components/StackEditContent';

import type { FlashbarProps } from '@cloudscape-design/components';
import type { OverviewTabAttr, OverviewTabInfo } from '../../../../contexts';

export type OverviewTabProps = OverviewTabAttr;

const stackEditValidator = z.object({
  name: z.string().min(1).max(256),
  description: z.string().max(256),
});

export type StackEditType = z.infer<typeof stackEditValidator>;

export type BuildOverviewTabNameProps = {
  stackName: string;
};
export function buildOverviewTabName({ stackName }: BuildOverviewTabNameProps): string {
  return stackName;
}

export const OverviewTab = (props: OverviewTabProps): JSX.Element => {
  const { modifyResourceTab } = useWorkspaceResourceContext();
  const [flashbarItems, setFlashbarItems] = useState<FlashbarProps.MessageDefinition[]>([]);

  // biome-ignore lint/correctness/useExhaustiveDependencies: <explanation>
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

  if (props.isEdit) {
    return (
      <StackEditContent
        tabId={props.tabId}
        stackId={props.stackId}
        stackName={props.stackName}
        stackDescription={props.description}
        flashbarItems={flashbarItems}
        setFlashbarItems={setFlashbarItems}
      />
    );
  }
  return (
    <StackDisplayContent
      tabId={props.tabId}
      stackId={props.stackId}
      stackName={props.stackName}
      stackDescription={props.description}
      flashbarItems={flashbarItems}
    />
  );
};
