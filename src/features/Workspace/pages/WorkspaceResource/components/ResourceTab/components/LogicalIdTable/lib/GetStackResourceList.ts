import { invoke } from '@tauri-apps/api/core';

import type { CommandResult } from '../../../../../../../../../lib/CommandResult';

export type GetStackResourceListProps = {
  stack_id: string;
  service_name: string;
  resource_name: string;
};

export async function getStackResourceList(props: GetStackResourceListProps): Promise<string[]> {
  const result = await invoke<CommandResult<string[]>>('get_stack_resource_list_command', props);
  if (!result.success) {
    throw new Error(result.value);
  }
  return result.value;
}
