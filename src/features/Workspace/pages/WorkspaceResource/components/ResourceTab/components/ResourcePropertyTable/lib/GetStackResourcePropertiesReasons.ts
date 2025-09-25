import { invoke } from '@tauri-apps/api/core';
import type { CommandResult } from '../../../../../../../../../lib/CommandResult';

export type GetStackResourcePropertiesReasonsProps = {
  stack_id: string;
  logical_id: string;
};

export async function getStackResourcePropertiesReasons(
  props: GetStackResourcePropertiesReasonsProps,
): Promise<Record<string, string>> {
  const result = await invoke<CommandResult<Record<string, string>>>(
    'get_stack_resource_properties_reasons_command',
    props,
  );
  if (!result.success) {
    throw new Error(result.value);
  }
  return result.value;
}
