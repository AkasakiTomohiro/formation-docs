import { invoke } from '@tauri-apps/api/core';

import type { CommandResult } from '../../../../../../../../../lib/CommandResult';

export async function getManualManagementResourceProperties(props: {
  resource_id: string;
  // biome-ignore lint/suspicious/noExplicitAny: <explanation>
}): Promise<Record<string, any>> {
  // biome-ignore lint/suspicious/noExplicitAny: <explanation>
  const result = await invoke<CommandResult<any>>('get_manual_resource_properties_command', props);
  if (!result.success) {
    throw new Error(result.value);
  }
  return result.value;
}
