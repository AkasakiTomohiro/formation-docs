import { invoke } from '@tauri-apps/api/core';
import type { CommandResult } from '../lib/CommandResult';
import type { Language } from './AppConfig';

export type GetTranslationCommand = {
  lang: Language;
  serviceName: string;
  resourceType: string;
};

export async function getTranslation(props: GetTranslationCommand): Promise<Record<string, string>> {
  if (props.lang === 'En') {
    return {};
  }
  const result = await invoke<CommandResult<Record<string, string>>>('get_translation_command', {
    lang: props.lang,
    service_name: props.serviceName,
    resource_type: props.resourceType,
  });
  if (result.success) {
    return result.value;
  }
  throw new Error(result.value);
}

export type SaveTranslationCommand = {
  lang: Language;
  serviceName: string;
  resourceType: string;
  translation: Record<string, string>;
};

export async function saveTranslation(props: SaveTranslationCommand): Promise<CommandResult<void>> {
  const result = await invoke<CommandResult<void>>('save_translation_command', {
    lang: props.lang,
    service_name: props.serviceName,
    resource_type: props.resourceType,
    translation: props.translation,
  });
  return result;
}

export type ExportTranslationZipCommand = {
  lang: Language;
};

export async function exportTranslationZip(props: ExportTranslationZipCommand): Promise<CommandResult<void>> {
  const result = await invoke<CommandResult<void>>('export_translation_file_command', {
    lang: props.lang,
  });
  return result;
}

export type ImportTranslationZipCommand = {
  lang: Language;
};

export async function importTranslationZip(props: ImportTranslationZipCommand): Promise<CommandResult<void>> {
  const result = await invoke<CommandResult<void>>('import_translation_file_command', {
    lang: props.lang,
  });
  return result;
}
