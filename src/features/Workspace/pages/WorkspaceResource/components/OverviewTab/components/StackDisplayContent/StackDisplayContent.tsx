import { useEffect, useState } from 'react';

import { useFlashbarContext } from '../../../../../../../../contexts/FlashbarContext';
import { useWorkspaceResourceContext } from '../../../../../../contexts';
import { StackDisplayContentPresentation } from './StackDisplayContent.presentation';
import { getStackOutputs } from './lib/GetStackOutputs';
import { getStackParameters } from './lib/GetStackParameters';

import type { OverviewTabInfo } from '../../../../../../contexts';
import type {
  StackDisplayContentPresentationProps,
  StackOutputsDisplayProps,
  StackParametersDisplayProps,
} from './StackDisplayContent.presentation';

export type StackContentProps = Pick<StackDisplayContentPresentationProps, 'stackName' | 'stackDescription'> & {
  /**
   * スタックのタブID
   */
  tabId: string;

  /**
   * スタックID
   */
  stackId: string;
};

export const StackDisplayContent = ({
  tabId,
  stackId,
  stackName,
  stackDescription,
}: StackContentProps): JSX.Element => {
  const { modifyResourceTab } = useWorkspaceResourceContext();
  const [stackParameters, setStackParameters] = useState<StackParametersDisplayProps[]>([]);
  const [stackOutputs, setStackOutputs] = useState<StackOutputsDisplayProps[]>([]);
  const { flashbarItems } = useFlashbarContext();

  // biome-ignore lint/correctness/useExhaustiveDependencies: <explanation>
  useEffect(() => {
    // スタックパラメータを取得し、表中に表示する
    getStackParameters({ stack_id: stackId }).then((stackParameters) => {
      const keys = Object.keys(stackParameters);
      const parameters: StackParametersDisplayProps[] = keys.map((key) => {
        return {
          name: key,
          type: stackParameters[key].Type,
          description: stackParameters[key].Description,
        };
      });
      setStackParameters(parameters);
    });

    // スタックのoutputを取得し、表中に表示する
    getStackOutputs({ stack_id: stackId }).then((stackOutputs) => {
      const outputs = stackOutputs.map((output) => {
        return {
          name: output.name,
          description: output.description,
          exportName: output.exportName,
          value: JSON.stringify(output.value), // FIXME: convertIntrinsicFunctionValue関数を使用した文字列を代入する
        };
      });
      setStackOutputs(outputs);
    });
  }, []);

  return (
    <StackDisplayContentPresentation
      stackName={stackName}
      stackDescription={stackDescription}
      stackParameters={stackParameters}
      stackOutputs={stackOutputs}
      flashbarItems={flashbarItems}
      onClickEdit={() => {
        modifyResourceTab(tabId, (originTab: OverviewTabInfo) => {
          return {
            ...originTab,
            isEdit: true,
          };
        });
      }}
    />
  );
};
