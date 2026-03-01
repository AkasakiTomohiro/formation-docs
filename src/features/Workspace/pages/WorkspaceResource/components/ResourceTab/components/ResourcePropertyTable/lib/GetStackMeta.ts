import { invoke } from '@tauri-apps/api/core';
import type { CommandResult } from '../../../../../../../../../lib/CommandResult';

export type GetStackMetaProps = {
  stack_id: string;
  logical_id: string;
};

type StackMeta = {
  reasons: Record<string, string>;
  description: string;
};

export async function getStackMeta(props: GetStackMetaProps): Promise<StackMeta> {
  const result = await invoke<CommandResult<StackMeta>>('get_stack_meta_command', props);
  if (!result.success) {
    throw new Error(result.value);
  }
  return result.value;
}
