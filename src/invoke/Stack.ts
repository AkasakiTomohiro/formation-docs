import { invoke } from '@tauri-apps/api/core';

import type { CommandResult } from '../lib/CommandResult';

export interface StackInfo {
  id: string;
  name: string;
  description_from_meta: string;
  description_from_stack?: string;
}

export async function loadStacks(): Promise<StackInfo[]> {
  const result = await invoke<CommandResult<StackInfo[]>>(
    'load_stacks_command',
  );
  if (!result.success) {
    throw new Error(result.value);
  }
  return result.value;
}

export async function deleteStack(stackId: string): Promise<void> {
  const result = await invoke<CommandResult<void>>('delete_stack_command', {
    stack_id: stackId,
  });
  if (!result.success) {
    throw new Error(result.value);
  }
  return result.value;
}

export async function importStack(stackFilePath: string): Promise<void> {
  const result = await invoke<CommandResult<void>>('import_stack_command', {
    stack_file_path: stackFilePath,
  });
  if (!result.success) {
    throw new Error(result.value);
  }
  return result.value;
}

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

export async function getStackResourceList(
  props: GetStackResourceListProps,
): Promise<string[]> {
  const result = await invoke<CommandResult<string[]>>(
    'get_stack_resource_list_command',
    props,
  );
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
  const result = await invoke<CommandResult<any>>(
    'get_stack_resource_properties_command',
    props,
  );
  if (!result.success) {
    throw new Error(result.value);
  }
  return result.value;
}

export type GetStackParametersProps = {
  stack_id: string;
};

export type GetStackParametersResult = Record<
  string,
  {
    Type: string;
    Description: string;
  }
>;

export async function getStackParameters(
  props: GetStackParametersProps,
): Promise<GetStackParametersResult> {
  const result = await invoke<CommandResult<GetStackParametersResult>>(
    'get_stack_parameters_command',
    props,
  );
  if (!result.success) {
    throw new Error(result.value);
  }
  return result.value;
}

export type GetStackOutputsProps = {
  stack_id: string;
};

type GetStackOutputResult = {
  [outputName: string]: {
    Value: string;
    Export?: {
      Name: string; // エクスポート名
    };
    Description?: string;
  };
};

export async function getStackOutputs(
  props: GetStackOutputsProps,
): Promise<GetStackOutputResult> {
  const result = await invoke<CommandResult<GetStackOutputResult>>(
    'get_stack_outputs_command',
    props,
  );
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
  const result = await invoke<CommandResult<any>>(
    'update_stack_meta_command',
    props,
  );
  if (!result.success) {
    throw new Error(result.value);
  }
  return result.value;
}

export type UpdateStackDetailProps = {
  stack_id: string;
  name: string;
  description: string;
};
export async function updateStackDetail(
  props: UpdateStackDetailProps,
  // biome-ignore lint/suspicious/noExplicitAny: <explanation>
): Promise<any> {
  // biome-ignore lint/suspicious/noExplicitAny: <explanation>
  const result = await invoke<CommandResult<any>>(
    'update_stack_detail_command',
    props,
  );
  if (!result.success) {
    throw new Error(result.value);
  }
  return result.value;
}
