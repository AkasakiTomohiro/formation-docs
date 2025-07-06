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
  const result = await invoke<CommandResult<any>>(
    'get_manual_management_resource_list_command',
    props,
  );
  if (!result.success) {
    throw new Error(result.value);
  }
  return result.value;
}

export type ManualManagementResourceSummary = {
  serviceName: string;
  recourseType: string[];
};

export async function loadManualManagementResourceSummary(): Promise<
  ManualManagementResourceSummary[]
> {
  // biome-ignore lint/suspicious/noExplicitAny: <explanation>
  const result = await invoke<CommandResult<any>>(
    'load_manual_management_resource_summary_command',
  );
  if (!result.success) {
    throw new Error(result.value);
  }
  return result.value;
}

export async function getManualManagementResourceProperties(props: {
  resource_id: string;
  // biome-ignore lint/suspicious/noExplicitAny: <explanation>
}): Promise<Record<string, any>> {
  // biome-ignore lint/suspicious/noExplicitAny: <explanation>
  const result = await invoke<CommandResult<any>>(
    'get_manual_resource_properties_command',
    props,
  );
  if (!result.success) {
    throw new Error(result.value);
  }
  return result.value;
}

export async function getManualManagementResourceReasons(props: {
  resource_id: string;
}): Promise<Record<string, string>> {
  // biome-ignore lint/suspicious/noExplicitAny: <explanation>
  const result = await invoke<CommandResult<any>>(
    'get_manual_resource_reasons_command',
    props,
  );
  if (!result.success) {
    throw new Error(result.value);
  }
  return result.value;
}

export async function updateManualResourceMeta(props: {
  resource_id: string;
  reasons: Record<string, string>;
  // biome-ignore lint/suspicious/noExplicitAny: <explanation>
}): Promise<any> {
  // biome-ignore lint/suspicious/noExplicitAny: <explanation>
  const result = await invoke<CommandResult<any>>(
    'update_manual_resource_meta_command',
    props,
  );
  if (!result.success) {
    throw new Error(result.value);
  }
  return result.value;
}
