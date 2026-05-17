import { createContext, useContext, useState } from 'react';
import { type AppConfig, getAppConfig } from '../invoke/AppConfig';

export type AppConfigContext = {
  loadAppConfig: () => Promise<void>;
  appConfig: AppConfig | null;
  setLanguage: (language: AppConfig['language']) => Promise<void>;
};

const AppConfigContext = createContext<AppConfigContext>({
  loadAppConfig: () => {
    throw new Error('loadAppConfig is not implemented');
  },
  appConfig: null,
  setLanguage: () => {
    throw new Error('setLanguage is not implemented');
  },
});

export function useAppConfigContext(): AppConfigContext {
  return useContext(AppConfigContext);
}

export interface AppConfigContextProviderProps {
  children: React.ReactNode;
}

export const AppConfigContextProvider = ({ children }: AppConfigContextProviderProps): JSX.Element => {
  const [appConfig, setAppConfig] = useState<AppConfigContext['appConfig']>(null);

  /**
   * AppConfig読み込み関数
   */
  const loadAppConfig = async (): Promise<void> => {
    try {
      const config = await getAppConfig();
      setAppConfig(config);
    } catch (error) {
      console.error('Failed to load app config:', error);
    }
  };

  /**
   *  AppConfigの言語設定を変更する関数
   * @param language "en" or "ja"
   */
  const setLanguage = async (language: AppConfig['language']): Promise<void> => {
    // TODO: AppConfigの言語設定を変更するinvoke関数を作成し呼び出す
    // 例: await invoke('set_app_config_language_command', { language });
    // 変更後、再度AppConfigを読み込む
    await loadAppConfig();
  };

  return (
    <AppConfigContext.Provider
      value={{
        loadAppConfig,
        appConfig,
        setLanguage,
      }}
    >
      {children}
    </AppConfigContext.Provider>
  );
};
