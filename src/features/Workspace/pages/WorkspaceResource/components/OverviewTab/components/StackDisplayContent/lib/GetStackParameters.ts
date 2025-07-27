import { invoke } from '@tauri-apps/api/core';

import type { CommandResult } from '../../../../../../../../../lib/CommandResult';

export type GetStackParametersProps = {
  stack_id: string;
};

export type GetStackParametersResult = Record<
  string,
  {
    Type: string;
    Description: string;
  }
>;

export async function getStackParameters(props: GetStackParametersProps): Promise<GetStackParametersResult> {
  const result = await invoke<CommandResult<GetStackParametersResult>>('get_stack_parameters_command', props);
  if (!result.success) {
    throw new Error(result.value);
  }
  return result.value;
}
