import { invoke } from '@tauri-apps/api/core';
import type { CommandResult } from '../../../../../../../../../lib/CommandResult';

export async function getManualManagementResourceReasons(props: {
  resource_id: string;
}): Promise<Record<string, string>> {
  const result = await invoke<CommandResult<any>>('get_manual_resource_reasons_command', props);
  if (!result.success) {
    throw new Error(result.value);
  }
  return result.value;
}
