import { invoke } from '@tauri-apps/api/core';

import type { CommandResult } from '../../../../lib/CommandResult';

export async function getAllStackOutputs(): Promise<Record<string, string>> {
  // biome-ignore lint/suspicious/noExplicitAny: <explanation>
  const result = await invoke<CommandResult<any>>('get_all_stack_outputs_command');
  if (!result.success) {
    throw new Error(result.value);
  }
  return result.value;
}
