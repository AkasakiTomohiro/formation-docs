import { invoke } from '@tauri-apps/api/core';
import type { CommandResult } from '../../../../../../../../../lib/CommandResult';

export type UpdateStackDetailProps = {
  stack_id: string;
  name: string;
  description: string;
};

export async function updateStackDetail(props: UpdateStackDetailProps): Promise<any> {
  const result = await invoke<CommandResult<any>>('update_stack_detail_command', props);
  if (!result.success) {
    throw new Error(result.value);
  }
  return result.value;
}
