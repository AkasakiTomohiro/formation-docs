import { invoke } from '@tauri-apps/api/core';
import type { CommandResult } from '../../../../../../../../../lib/CommandResult';

export async function updateManualResourceProperties(props: {
  resource_id: string;
  properties: string;
}): Promise<void> {
  const result = await invoke<CommandResult<any>>('update_manual_resource_properties_command', props);
  if (!result.success) {
    throw new Error(result.value);
  }
}
