import { invoke } from '@tauri-apps/api/core';
import type { CommandResult } from '../../../../../../../../../lib/CommandResult';

export type GetStackOutputsProps = {
  stack_id: string;
};

type GetStackOutputResult = {
  name: string;
  description: string | null;
  exportName: string | null;
  value: any;
};

export async function getStackOutputs(props: GetStackOutputsProps): Promise<GetStackOutputResult[]> {
  const result = await invoke<CommandResult<GetStackOutputResult[]>>('get_stack_outputs_command', props);
  if (!result.success) {
    throw new Error(result.value);
  }
  return result.value;
}
