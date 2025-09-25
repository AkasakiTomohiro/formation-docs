import { invoke } from '@tauri-apps/api/core';
import type { CommandResult } from '../../../../../../../../../lib/CommandResult';

export type UpdateStackMetaProps = {
  stack_id: string;
  logical_id: string;
  reasons: Record<string, string>;
};

export async function updateStackMeta(props: UpdateStackMetaProps): Promise<any> {
  const result = await invoke<CommandResult<any>>('update_stack_meta_command', props);
  if (!result.success) {
    throw new Error(result.value);
  }
  return result.value;
}
