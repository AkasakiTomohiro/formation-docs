import { createContext, useCallback, useContext, useState } from 'react';
import { useLoaderData, useNavigate } from 'react-router';
import { getAllStackOutputs } from './lib/GetAllStackOutputs';
import { loadManualManagementResourceSummary } from './lib/LoadManualManagementResourceSummary';
import { loadTemplateSummary } from './lib/LoadTemplateSummary';
import type { TokenGroupProps } from '@cloudscape-design/components';
import type { WorkspaceLayoutLoaderData } from '../Loader';
import type { TemplateSummary } from './lib/LoadTemplateSummary';

type TabInfo<TType extends string, T extends { tabId: string } = { tabId: string }> = {
  type: TType;
} & T;

export type OverviewTabAttr = {
  tabId: string;
  stackId: string;
  stackName: string;
  description: string;
  editingValues?: {
    stackName: string;
    stackDescription: string;
  };
};
export type ResourceTabAttr = {
  tabId: string;
  stackId: string;
  stackName: string;
  serviceName: string;
  resourceName: string;
  selectedLogicalId?: string;
};
export type ManualResourceTabAttr = {
  tabId: string;
  serviceName: string;
  resourceName: string;
  selectedResourceId?: string;
};

export type OverviewTabInfo = TabInfo<'overview', OverviewTabAttr>;
export type ResourceTabInfo = TabInfo<'resource', ResourceTabAttr>;
export type ManualOverviewTabInfo = TabInfo<'manualOverview'>;
export type ManualResourceTabInfo = TabInfo<'manualResource', ManualResourceTabAttr>;

export type WorkspaceTabInfo = OverviewTabInfo | ResourceTabInfo | ManualOverviewTabInfo | ManualResourceTabInfo;

export const ManualManagementId = 'manualManagement';

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
  resourceTabs: WorkspaceTabInfo[];
  setResourceTabs: React.Dispatch<React.SetStateAction<WorkspaceTabInfo[]>>;

  /**
   * リソースタブ追加関数
   * @param tab - 追加するタブ情報
   */
  addResourceTab: (tab: WorkspaceTabInfo) => void;

  /**
   * リソースタブ変更関数
   * @param tabId - 変更するタブID
   * @param modifyFn - タブ情報を変更する関数
   */
  modifyResourceTab: <T extends WorkspaceTabInfo = WorkspaceTabInfo>(
    tabId: string,
    modifyFn: (originTab: T) => T,
  ) => void;

  /**
   * リソースタブ削除関数
   * @param tabId - 削除するタブID
   */
  deleteResourceTab: (tabId: string) => void;

  /**
   * リソースタブ全削除関数
   */
  deleteAllResourceTabs: () => void;

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

  /**
   * 全スタックのOutputs
   */
  allStackOutputs: Record<string, string>;

  /**
   * 全スタックのOutputs取得関数
   */
  loadAllStackOutputs: () => Promise<void>;
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
  deleteResourceTab: () => {
    throw new Error('deleteResourceTab is not implemented');
  },
  deleteAllResourceTabs: () => {
    throw new Error('deleteAllResourceTabs is not implemented');
  },
  activeTabId: undefined,
  setActiveTabId: () => {
    throw new Error('setActiveTabId is not implemented');
  },
  sideMenu: [],
  loadSideMenu: () => {
    throw new Error('loadSideMenu is not implemented');
  },
  allStackOutputs: {},
  loadAllStackOutputs: () => {
    throw new Error('loadAllStackOutputs is not implemented');
  },
});

export function useWorkspaceResourceContext(): WorkspaceResourceContext {
  return useContext(WorkspaceResourceContext);
}

export interface WorkspaceResourceProviderProps {
  children?: React.ReactNode;
}

export const WorkspaceResourceProvider = ({ children }: WorkspaceResourceProviderProps): JSX.Element => {
  const workspace = useLoaderData<WorkspaceLayoutLoaderData>();
  const navigate = useNavigate();

  // 検索
  const [searchValue, setSearchValue] = useState<string>('');
  const [tokenGroup, setTokenGroup] = useState<TokenGroupProps.Item[]>([]);

  // WorkspaceResourceタブ
  const [resourceTabs, setResourceTabs] = useState<WorkspaceTabInfo[]>([]);
  const [activeTabId, setActiveTabId] = useState<string | undefined>(undefined);

  // サイドメニュー
  const [sideMenu, setSideMenu] = useState<TemplateSummary[]>([]);

  // 全スタックのOutputs
  const [allStackOutputs, setAllStackOutputs] = useState<Record<string, string>>({});

  // サイドメニュー要素取得関数
  const loadSideMenu = useCallback(() => {
    return Promise.all([loadTemplateSummary(), loadManualManagementResourceSummary()]).then(
      ([templateSummary, manualManagementSummary]) => {
        setSideMenu([
          ...templateSummary,
          {
            id: ManualManagementId,
            sectionGroupName: '手動管理リソース',
            resources: manualManagementSummary,
          },
        ]);
      },
    );
  }, []);

  // 全スタックのOutputs取得関数
  const loadAllStackOutputs = useCallback(() => {
    return getAllStackOutputs().then((outputs) => {
      setAllStackOutputs(outputs);
    });
  }, []);

  // リソースタブ追加関数
  const addResourceTab = useCallback((newTab: WorkspaceTabInfo) => {
    setResourceTabs((prev) => {
      const existingTab = prev.find((tab) => tab.tabId === newTab.tabId);
      if (existingTab) {
        if (
          newTab.type === 'resource' &&
          existingTab.type === 'resource' &&
          existingTab.selectedLogicalId !== newTab.selectedLogicalId
        ) {
          // type='resource'で、selectedLogicalIdが異なる場合は更新する
          return prev.map((tab) => (tab.tabId === newTab.tabId ? newTab : tab));
        }
        return prev;
      }
      return [...prev, newTab];
    });
    setActiveTabId(newTab.tabId);
  }, []);

  // リソースタブ変更関数
  const modifyResourceTab = useCallback(
    <T extends WorkspaceTabInfo = WorkspaceTabInfo>(tabId: string, modifyFn: (originTab: T) => T) => {
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

  // リソースタブ全削除関数
  const deleteAllResourceTabs = useCallback(() => {
    setResourceTabs([]);
    setActiveTabId(undefined);
    navigate(`/workspaces/${workspace.id}`);
  }, [navigate, workspace.id]);

  // リソースタブ削除関数
  const deleteResourceTab = useCallback(
    (tabId: string) => {
      const filteredTabs = resourceTabs.filter((tab) => tab.tabId !== tabId);
      if (filteredTabs.length === 0) {
        deleteAllResourceTabs();
        return;
      }
      setResourceTabs(filteredTabs);
      if (activeTabId === tabId) {
        setActiveTabId(filteredTabs[filteredTabs.length - 1]?.tabId);
      }
    },
    [resourceTabs, activeTabId, deleteAllResourceTabs],
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
        deleteResourceTab,
        deleteAllResourceTabs,
        activeTabId,
        setActiveTabId,
        sideMenu,
        loadSideMenu,
        allStackOutputs,
        loadAllStackOutputs,
      }}
    >
      {children}
    </WorkspaceResourceContext.Provider>
  );
};
