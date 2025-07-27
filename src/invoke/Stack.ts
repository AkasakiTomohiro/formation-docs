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

export type GetStackResourceListProps = {
  stack_id: string;
  service_name: string;
  resource_name: string;
};

export async function getStackResourceList(props: GetStackResourceListProps): Promise<string[]> {
  const result = await invoke<CommandResult<string[]>>('get_stack_resource_list_command', props);
  if (!result.success) {
    throw new Error(result.value);
  }
  return result.value;
}

export type GetStackResourcePropertiesProps = {
  stack_id: string;
  logical_id: string;
};

export async function getStackResourceProperties(
  props: GetStackResourcePropertiesProps,
  // biome-ignore lint/suspicious/noExplicitAny: <explanation>
): Promise<any> {
  // biome-ignore lint/suspicious/noExplicitAny: <explanation>
  const result = await invoke<CommandResult<any>>('get_stack_resource_properties_command', props);
  if (!result.success) {
    throw new Error(result.value);
  }
  return result.value;
}

export type GetStackResourcePropertiesReasonsProps = {
  stack_id: string;
  logical_id: string;
};

export async function getStackResourcePropertiesReasons(
  props: GetStackResourcePropertiesReasonsProps,
): Promise<Record<string, string>> {
  const result = await invoke<CommandResult<Record<string, string>>>(
    'get_stack_resource_properties_reasons_command',
    props,
  );
  if (!result.success) {
    throw new Error(result.value);
  }
  return result.value;
}

export type UpdateStackMetaProps = {
  stack_id: string;
  logical_id: string;
  reasons: Record<string, string>;
};
export async function updateStackMeta(
  props: UpdateStackMetaProps,
  // biome-ignore lint/suspicious/noExplicitAny: <explanation>
): Promise<any> {
  // biome-ignore lint/suspicious/noExplicitAny: <explanation>
  const result = await invoke<CommandResult<any>>('update_stack_meta_command', props);
  if (!result.success) {
    throw new Error(result.value);
  }
  return result.value;
}
