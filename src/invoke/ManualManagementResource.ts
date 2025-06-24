import { invoke } from '@tauri-apps/api/core';

import type { CommandResult } from '../lib/CommandResult';

export type NewManualManagementResourceProperties = {
  resource_id: string;
  description: string;
  service_name: string;
  resource_name: string;
};

export async function newManualManagementResource(
  props: NewManualManagementResourceProperties,
  // biome-ignore lint/suspicious/noExplicitAny: <explanation>
): Promise<any> {
  // biome-ignore lint/suspicious/noExplicitAny: <explanation>
  const result = await invoke<CommandResult<any>>(
    'new_manual_management_resource_command',
    props,
  );
  if (!result.success) {
    throw new Error(result.value);
  }
  return result.value;
}

export type ManualManagementResource = {
  resourceId: string;
  type: string;
  description: string;
};

export async function getManualManagementResourceList(): Promise<
  ManualManagementResource[]
> {
  // biome-ignore lint/suspicious/noExplicitAny: <explanation>
  const result = await invoke<CommandResult<any>>(
    'get_manual_management_resource_list_command',
  );
  if (!result.success) {
    throw new Error(result.value);
  }
  return result.value;
}
