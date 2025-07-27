import { invoke } from '@tauri-apps/api/core';

import type { StackInfo } from '../features/Workspace/lib/StackInfo';
import type { CommandResult } from '../lib/CommandResult';

export async function loadStack(stackName: string): Promise<StackInfo> {
  const result = await invoke<CommandResult<StackInfo>>('load_stack_command', {
    stack_id: stackName,
  });
  if (!result.success) {
    throw new Error(result.value);
  }
  return result.value;
}
