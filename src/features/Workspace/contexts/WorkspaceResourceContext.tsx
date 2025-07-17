import { createContext, useCallback, useContext, useState } from 'react';

import { ManualManagementId } from '../lib/FilterSideMenu';
import { loadManualManagementResourceSummary } from './lib/LoadManualManagementResourceSummary';
import { loadTemplateSummary } from './lib/LoadTemplateSummary';

import type { WorkspaceResourceInfo } from '../pages';
import type { TemplateSummary } from './lib/LoadTemplateSummary';

import type { TokenGroupProps } from '@cloudscape-design/components';

export type TabInfo<
  TType extends string,
  T extends { tabId: string } = { tabId: string },
> = {
  type: TType;
} & T;

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
   * リソースタブ追加関数
   * @param tab - 追加するタブ情報
   */
  addResourceTab: (tab: WorkspaceResourceInfo) => void;

  /**
   * リソースタブ変更関数
   * @param tabId - 変更するタブのID
   * @param modifyFn - タブ情報を変更する関数
   */
  modifyResourceTab: <T extends WorkspaceResourceInfo = WorkspaceResourceInfo>(
    tabId: string,
    modifyFn: (originTab: T) => T,
  ) => void;

  /**
   * アクティブタブID
   */
  activeTabId: string | undefined;
  setActiveTabId: React.Dispatch<React.SetStateAction<string | undefined>>;

  /**
   * サイドメニュー要素
   */
  sideMenu: TemplateSummary[];

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
  addResourceTab: () => {
    throw new Error('addResourceTab is not implemented');
  },
  modifyResourceTab: () => {
    throw new Error('modifyResourceTab is not implemented');
  },
  activeTabId: undefined,
  setActiveTabId: () => {
    throw new Error('setActiveTabId is not implemented');
  },
  sideMenu: [],
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

  // リソースタブ追加関数
  const addResourceTab = useCallback((newTab: WorkspaceResourceInfo) => {
    setResourceTabs((prev) => {
      const existingTab = prev.find((tab) => tab.tabId === newTab.tabId);
      if (existingTab) {
        return prev;
      }
      return [...prev, newTab];
    });
    setActiveTabId(newTab.tabId);
  }, []);

  // リソースタブ変更関数
  const modifyResourceTab = useCallback(
    <T extends WorkspaceResourceInfo = WorkspaceResourceInfo>(
      tabId: string,
      modifyFn: (originTab: T) => T,
    ) => {
      setResourceTabs((prev) =>
        prev.map((tab) => {
          if (tab.tabId === tabId) {
            return modifyFn(tab as T);
          }
          return tab;
        }),
      );
    },
    [],
  );

  return (
    <WorkspaceResourceContext.Provider
      value={{
        searchValue,
        setSearchValue,
        tokenGroup,
        setTokenGroup,
        resourceTabs,
        setResourceTabs,
        addResourceTab,
        modifyResourceTab,
        activeTabId,
        setActiveTabId,
        sideMenu,
        loadSideMenu,
      }}
    >
      {children}
    </WorkspaceResourceContext.Provider>
  );
};
