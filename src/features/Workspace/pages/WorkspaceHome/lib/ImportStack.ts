import { invoke } from '@tauri-apps/api/core';

import type { CommandResult } from '../../../../../lib/CommandResult';

export async function importStack(stackFilePath: string): Promise<void> {
  const result = await invoke<CommandResult<void>>('import_stack_command', {
    stack_file_path: stackFilePath,
  });
  if (!result.success) {
    throw new Error(result.value);
  }
  return result.value;
}
