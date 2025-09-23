import { invoke } from '@tauri-apps/api/core';
import type { CommandResult } from '../../../../../../../lib/CommandResult';
import type { StackInfo } from '../../../../../lib/StackInfo';

export async function loadStack(stackName: string): Promise<StackInfo> {
  const result = await invoke<CommandResult<StackInfo>>('load_stack_command', {
    stack_id: stackName,
  });
  if (!result.success) {
    throw new Error(result.value);
  }
  return result.value;
}
