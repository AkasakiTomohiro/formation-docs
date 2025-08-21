import { invoke } from '@tauri-apps/api/core';

import type { CommandResult } from '../../../../../../../../../lib/CommandResult';

export type LoadParameterAndResourceListProps = {
  stack_id: string;
};
export type LoadParameterAndResourceListResult = {
  parameters: string[];
  resources: Record<
    string,
    {
      serviceName: string;
      serviceType: string;
    }
  >;
};

export async function loadParameterAndResourceList(
  props: LoadParameterAndResourceListProps,
): Promise<LoadParameterAndResourceListResult> {
  const result = await invoke<CommandResult<LoadParameterAndResourceListResult>>(
    'load_parameter_and_resource_list_command',
    props,
  );
  if (!result.success) {
    throw new Error(result.value);
  }
  return result.value;
}
