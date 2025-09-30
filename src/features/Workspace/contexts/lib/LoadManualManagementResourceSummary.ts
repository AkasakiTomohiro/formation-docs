import { invoke } from '@tauri-apps/api/core';
import type { CommandResult } from '../../../../lib/CommandResult';

export type ManualManagementResourceSummary = {
  serviceName: string;
  recourseType: string[];
};

export async function loadManualManagementResourceSummary(): Promise<ManualManagementResourceSummary[]> {
  const result = await invoke<CommandResult<any>>('load_manual_management_resource_summary_command');
  if (!result.success) {
    throw new Error(result.value);
  }
  return result.value;
}
