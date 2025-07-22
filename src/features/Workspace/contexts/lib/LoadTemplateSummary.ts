import { invoke } from '@tauri-apps/api/core';

import type { CommandResult } from '../../../../lib/CommandResult';

export type TemplateSummary = {
  id: string;
  sectionGroupName: string;
  resources: {
    serviceName: string;
    recourseType: string[];
  }[];
};

export async function loadTemplateSummary(): Promise<TemplateSummary[]> {
  const result = await invoke<CommandResult<TemplateSummary[]>>(
    'load_template_summary_command',
  );
  if (!result.success) {
    throw new Error(result.value);
  }
  return result.value;
}
