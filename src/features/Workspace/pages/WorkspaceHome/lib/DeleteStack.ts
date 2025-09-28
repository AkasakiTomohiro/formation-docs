import { invoke } from '@tauri-apps/api/core';
import type { CommandResult } from '../../../../../lib/CommandResult';

export async function deleteStack(stackId: string): Promise<void> {
  const result = await invoke<CommandResult<void>>('delete_stack_command', {
    stack_id: stackId,
  });
  if (!result.success) {
    throw new Error(result.value);
  }
  return result.value;
}
