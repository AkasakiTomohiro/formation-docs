import { invoke } from '@tauri-apps/api/core';

import type { CommandResult } from '../../../../../../../../../lib/CommandResult';

export type GetStackResourcePropertiesProps = {
  stack_id: string;
  logical_id: string;
};

export async function getStackResourceProperties(
  props: GetStackResourcePropertiesProps,
  // biome-ignore lint/suspicious/noExplicitAny: <explanation>
): Promise<any> {
  // biome-ignore lint/suspicious/noExplicitAny: <explanation>
  const result = await invoke<CommandResult<any>>('get_stack_resource_properties_command', props);
  if (!result.success) {
    throw new Error(result.value);
  }
  return result.value;
}
