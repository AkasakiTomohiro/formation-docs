import { invoke } from '@tauri-apps/api/core';
import type { CommandResult } from '../../../../../../../../../lib/CommandResult';

export async function getAWSServiceList(): Promise<Record<string, string[]>> {
  const result = await invoke<CommandResult<Record<string, string[]>>>('get_aws_service_list_command');
  if (!result.success) {
    throw new Error(result.value);
  }
  return result.value;
}
