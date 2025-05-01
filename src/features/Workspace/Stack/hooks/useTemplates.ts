import { useEffect, useState } from 'react';

import { loadTemplateSummary } from '../../../../invoke/Stack';

export type TemplateSummary = {
  id: string;
  stackName: string;
  resources: {
    serviceName: string;
    recourseType: string[];
  }[];
};

export type UseTemplatesResult = {
  sideMenu: TemplateSummary[];
};

export function useTemplates(): UseTemplatesResult {
  const [sideMenu, setSideMenu] = useState<UseTemplatesResult['sideMenu']>([]);

  useEffect(() => {
    loadTemplateSummary().then((summary) => {
      setSideMenu(summary);
    });
  }, []);
  // テンプレート一覧を取得
  return { sideMenu };
}
