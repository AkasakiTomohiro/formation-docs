import { useEffect, useState } from 'react';

import { useWorkspaceResourceContext } from '../../../../../../contexts';
import { convertIntrinsicFunctionValue, isIntrinsicFunction } from '../../../../lib/CreateResourceTableItems';
import { loadParameterAndResourceList } from '../../../ResourceTab/components/ResourcePropertyTable/lib/LoadParameterAndResourceList';
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
  const { modifyResourceTab, addResourceTab } = useWorkspaceResourceContext();
  const [stackParameters, setStackParameters] = useState<StackParametersDisplayProps[]>([]);
  const [stackOutputs, setStackOutputs] = useState<StackOutputsDisplayProps[]>([]);

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
    Promise.all([getStackOutputs({ stack_id: stackId }), loadParameterAndResourceList({ stack_id: stackId })]).then(
      ([stackOutputs, parameterAndResourceList]) => {
        const outputs = stackOutputs.map((output) => {
          const intrinsic = isIntrinsicFunction(output.value);
          let value: StackOutputsDisplayProps['value'];
          if (intrinsic === undefined) {
            value = {
              type: 'value',
              value: output.value,
            };
          } else {
            value = convertIntrinsicFunctionValue(intrinsic, output.value, {
              stackId: stackId,
              stackName: stackName,
              parameters: parameterAndResourceList.parameters,
              resources: parameterAndResourceList.resources,
              externalResources: {},
            });
          }
          return {
            name: output.name,
            description: output.description,
            exportName: output.exportName,
            value: value,
          };
        });
        setStackOutputs(outputs);
      },
    );
  }, []);

  return (
    <StackDisplayContentPresentation
      stackName={stackName}
      stackDescription={stackDescription}
      stackParameters={stackParameters}
      stackOutputs={stackOutputs}
      onClickEdit={() => {
        modifyResourceTab(tabId, (originTab: OverviewTabInfo) => {
          return {
            ...originTab,
            isEdit: true,
          };
        });
      }}
      onClickValueLink={(item) => () => addResourceTab(item)}
    />
  );
};
