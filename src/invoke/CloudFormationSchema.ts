import { invoke } from '@tauri-apps/api/core';

import type { CommandResult } from '../lib/CommandResult';

export async function getCloudFormationSchema(
  service_name: string,
  resource_name: string,
): Promise<string> {
  const result = await invoke<CommandResult<string>>(
    'get_cloudformation_schema_command',
    {
      service_name: service_name,
      resource_name: resource_name,
    },
  );
  if (!result.success) {
    throw new Error(result.value);
  }
  return result.value;
}

export type AWSService = {
  service_name: string;
  resources: string[];
};

export async function getAWSServiceList(): Promise<AWSService[]> {
  const result = await invoke<CommandResult<AWSService[]>>(
    'get_aws_service_list_command',
  );
  if (!result.success) {
    throw new Error(result.value);
  }
  return result.value;
}
