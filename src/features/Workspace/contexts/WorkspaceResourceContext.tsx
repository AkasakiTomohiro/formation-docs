import { createContext, useCallback, useContext, useState } from 'react';

import { loadManualManagementResourceSummary } from '../../../invoke/ManualManagementResource';
import { loadTemplateSummary } from '../../../invoke/Stack';
import { ManualManagementId } from '../lib/FilterSideMenu';

import type { TemplateSummary } from '../../../invoke/Stack';
import type { WorkspaceResourceInfo } from '../pages';

import type { TokenGroupProps } from '@cloudscape-design/components';

export interface WorkspaceResourceContext {
  /**
   * 検索ワード
   */
  searchValue: string;
  setSearchValue: React.Dispatch<React.SetStateAction<string>>;

  /**
   * 検索トークン
   */
  tokenGroup: TokenGroupProps.Item[];
  setTokenGroup: React.Dispatch<React.SetStateAction<TokenGroupProps.Item[]>>;

  /**
   * リソースタブ
   */
  resourceTabs: WorkspaceResourceInfo[];
  setResourceTabs: React.Dispatch<
    React.SetStateAction<WorkspaceResourceInfo[]>
  >;

  /**
   * アクティブタブID
   */
  activeTabId: string | undefined;
  setActiveTabId: React.Dispatch<React.SetStateAction<string | undefined>>;

  /**
   * サイドメニュー要素
   */
  sideMenu: TemplateSummary[];
  setSideMenu: React.Dispatch<React.SetStateAction<TemplateSummary[]>>;

  /**
   * サイドメニュー要素取得関数
   */
  loadSideMenu: () => Promise<void>;
}

const WorkspaceResourceContext = createContext<WorkspaceResourceContext>({
  searchValue: '',
  setSearchValue: () => {
    throw new Error('setSearchValue is not implemented');
  },
  tokenGroup: [],
  setTokenGroup: () => {
    throw new Error('setTokenGroup is not implemented');
  },
  resourceTabs: [],
  setResourceTabs: () => {
    throw new Error('setResourceTabs is not implemented');
  },
  activeTabId: undefined,
  setActiveTabId: () => {
    throw new Error('setActiveTabId is not implemented');
  },
  sideMenu: [],
  setSideMenu: () => {
    throw new Error('setSideMenu is not implemented');
  },
  loadSideMenu: () => {
    throw new Error('loadSideMenu is not implemented');
  },
});

export function useWorkspaceResourceContext(): WorkspaceResourceContext {
  return useContext(WorkspaceResourceContext);
}

export interface WorkspaceResourceProviderProps {
  children?: React.ReactNode;
}

export const WorkspaceResourceProvider = ({
  children,
}: WorkspaceResourceProviderProps): JSX.Element => {
  // 検索
  const [searchValue, setSearchValue] = useState<string>('');
  const [tokenGroup, setTokenGroup] = useState<TokenGroupProps.Item[]>([]);

  // WorkspaceResourceタブ
  const [resourceTabs, setResourceTabs] = useState<WorkspaceResourceInfo[]>([]);
  const [activeTabId, setActiveTabId] = useState<string | undefined>(undefined);

  // サイドメニュー
  const [sideMenu, setSideMenu] = useState<TemplateSummary[]>([]);

  // サイドメニュー要素取得関数
  const loadSideMenu = useCallback(() => {
    return Promise.all([
      loadTemplateSummary(),
      loadManualManagementResourceSummary(),
    ]).then(([templateSummary, manualManagementSummary]) => {
      setSideMenu([
        ...templateSummary,
        {
          id: ManualManagementId,
          sectionGroupName: '手動管理リソース',
          resources: manualManagementSummary,
        },
      ]);
    });
  }, []);

  return (
    <WorkspaceResourceContext.Provider
      value={{
        searchValue,
        setSearchValue,
        tokenGroup,
        setTokenGroup,
        resourceTabs,
        setResourceTabs,
        activeTabId,
        setActiveTabId,
        sideMenu,
        setSideMenu,
        loadSideMenu,
      }}
    >
      {children}
    </WorkspaceResourceContext.Provider>
  );
};
