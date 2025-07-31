import { invoke } from '@tauri-apps/api/core';

import type { CommandResult } from '../../../../../../../../../lib/CommandResult';

export async function updateManualResourceMeta(props: {
  resource_id: string;
  reasons: Record<string, string>;
  // biome-ignore lint/suspicious/noExplicitAny: <explanation>
}): Promise<any> {
  // biome-ignore lint/suspicious/noExplicitAny: <explanation>
  const result = await invoke<CommandResult<any>>('update_manual_resource_meta_command', props);
  if (!result.success) {
    throw new Error(result.value);
  }
  return result.value;
}
