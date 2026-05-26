import { createContext, useCallback, useContext, useEffect, useState } from 'react';
import { type AppConfig, getAppConfig, updateAppConfigLanguage } from '../invoke/AppConfig';

export type AppConfigContext = {
  loadAppConfig: () => Promise<void>;
  appConfig: AppConfig | null;
  setLanguage: (language: AppConfig['language']) => Promise<boolean>;
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
  const loadAppConfig = useCallback(async (): Promise<void> => {
    try {
      const config = await getAppConfig();
      setAppConfig(config);
    } catch (error) {
      console.error('Failed to load app config:', error);
    }
  }, []);

  /**
   *  AppConfigの言語設定を変更する関数
   */
  const setLanguage = async (language: AppConfig['language']): Promise<boolean> => {
    try {
      console.log('Updating app config language to:', language);
      await updateAppConfigLanguage(language);
    } catch (error) {
      console.error('Failed to update app config language:', error);
      return false;
    }
    // 変更後、再度AppConfigを読み込む
    await loadAppConfig();
    return true;
  };

  useEffect(() => {
    loadAppConfig();
  }, [loadAppConfig]);

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
