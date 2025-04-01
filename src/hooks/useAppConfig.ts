import { useCallback, useEffect, useRef, useState } from 'react';
import { useNavigate } from 'react-router';

import { loadAppConfig, setupApp } from '../invoke/AppConfig';

import type { AppConfig } from '../invoke/AppConfig';

export type UseAppConfigResult = {
  /**
   * AppConfigの読み込み状態
   */
  appConfig: AppConfig | null;
};

export function useAppConfig(): UseAppConfigResult {
  const isFirstRender = useRef(true);
  const [appConfig, setAppConfig] = useState<AppConfig | null>(null);
  const navigate = useNavigate();

  // biome-ignore lint/correctness/useExhaustiveDependencies: <explanation>
  const loadAppConfigWrap = useCallback(async () => {
    const config = await loadAppConfig();
    setAppConfig(config);
    if (config.initialized === false) {
      await setupApp().then(() => {
        navigate('/workspaces');
      });
    } else {
      navigate('/workspaces');
    }
  }, []);

  useEffect(() => {
    if (isFirstRender.current) {
      isFirstRender.current = false;
      return;
    }
    loadAppConfigWrap().catch((error) => console.log(error));
  }, [loadAppConfigWrap]);

  return { appConfig };
}
