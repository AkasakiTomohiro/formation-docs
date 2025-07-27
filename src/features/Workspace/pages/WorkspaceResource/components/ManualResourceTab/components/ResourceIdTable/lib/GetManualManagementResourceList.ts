import { invoke } from '@tauri-apps/api/core';

import type { CommandResult } from '../../../../../../../../../lib/CommandResult';

export type ManualManagementResourceListProps = {
  service_name: string;
  resource_name: string;
};
export type ManualManagementResource = {
  resourceId: string;
  type: string;
  description: string;
};

export async function getManualManagementResourceList(
  props: ManualManagementResourceListProps,
): Promise<ManualManagementResource[]> {
  // biome-ignore lint/suspicious/noExplicitAny: <explanation>
  const result = await invoke<CommandResult<any>>('get_manual_management_resource_list_command', props);
  if (!result.success) {
    throw new Error(result.value);
  }
  return result.value;
}
