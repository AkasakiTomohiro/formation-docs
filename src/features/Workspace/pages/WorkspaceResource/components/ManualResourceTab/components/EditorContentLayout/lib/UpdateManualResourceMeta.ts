import { invoke } from '@tauri-apps/api/core';
import type { CommandResult } from '../../../../../../../../../lib/CommandResult';

export async function updateManualResourceMeta(props: {
  resource_id: string;
  reasons?: Record<string, string>;
  description?: string;
}): Promise<any> {
  const result = await invoke<CommandResult<any>>('update_manual_resource_meta_command', { update_info: props });
  if (!result.success) {
    throw new Error(result.value);
  }
  return result.value;
}
