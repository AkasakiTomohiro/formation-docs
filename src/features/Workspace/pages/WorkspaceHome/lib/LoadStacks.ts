import { invoke } from '@tauri-apps/api/core';
import type { CommandResult } from '../../../../../lib/CommandResult';
import type { StackInfo } from '../../../lib/StackInfo';

export async function loadStacks(): Promise<StackInfo[]> {
  const result = await invoke<CommandResult<StackInfo[]>>('load_stacks_command');
  if (!result.success) {
    throw new Error(result.value);
  }
  return result.value;
}
