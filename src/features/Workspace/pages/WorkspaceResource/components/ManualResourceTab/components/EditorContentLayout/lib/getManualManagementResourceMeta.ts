import { invoke } from '@tauri-apps/api/core';
import type { CommandResult } from '../../../../../../../../../lib/CommandResult';

type ManualManagementResourceMeta = {
  reasons: Record<string, string>;
  description: string;
};
export async function getManualManagementResourceMeta(props: {
  resource_id: string;
}): Promise<ManualManagementResourceMeta> {
  const result = await invoke<CommandResult<any>>('get_manual_resource_meta_command', props);
  if (!result.success) {
    throw new Error(result.value);
  }
  return result.value;
}
