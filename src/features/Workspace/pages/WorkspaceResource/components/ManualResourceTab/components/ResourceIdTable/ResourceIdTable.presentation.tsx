import { Box, Button, Container, Header, Link, SpaceBetween, Table, TextFilter } from '@cloudscape-design/components';
import type { ButtonProps, LinkProps, TextFilterProps } from '@cloudscape-design/components';
import type { ManualManagementResource } from './lib/GetManualManagementResourceList';
export type ResourceIdTablePresentationProps = {
  /**
   * フィルタリングテキスト
   */
  filteringText: string;

  /**
   * スタックのタブID
   */
  tabId: string;

  /**
   * 選択されているリソースID
   */
  selectedResourceId?: string;

  /**
   * リソースのリスト
   */
  resourceList: ManualManagementResource[];

  /**
   * リソースIDクリック時のイベントハンドラ
   */
  onClickResourceId: (item: ManualManagementResource) => LinkProps['onClick'];

  /**
   * 開く閉じるボタン押下
   */
  setToggleOpen: ButtonProps['onClick'];

  /**
   * 論理IDテーブルの拡大ボタン押下時のイベントハンドラ
   */
  onClickExpand: ButtonProps['onClick'];

  /**
   * フィルタリングテキストの変更イベントハンドラ
   */
  onChangeFilteringText?: TextFilterProps['onChange'];

  /**
   * ローディング中かどうか
   */
  isLoading: boolean;

  /**
   * リソースIDテーブルを開いているかどうか
   */
  isOpen: boolean;
};

export const ResourceIdTablePresentation = ({
  filteringText,
  selectedResourceId,
  resourceList,
  onClickResourceId,
  setToggleOpen,
  onClickExpand,
  onChangeFilteringText,
  isLoading,
  isOpen,
}: ResourceIdTablePresentationProps) => {
  if (isOpen) {
    return (
      <div
        style={{
          width: selectedResourceId === undefined ? '100%' : '300px',
          minWidth: '300px',
          height: '100%',
        }}
      >
        <Table
          columnDefinitions={[
            {
              id: 'resourceId',
              header: 'リソースID',
              cell: (item) => (
                <Link href="#" onClick={onClickResourceId(item)}>
                  {item.resourceId}
                </Link>
              ),
              sortingField: 'name',
              isRowHeader: true,
            },
          ]}
          enableKeyboardNavigation
          items={resourceList}
          loadingText="リソースを読み込み中..."
          loading={isLoading}
          sortingDisabled
          empty={
            <Box margin={{ vertical: 'xs' }} textAlign="center" color="inherit">
              <SpaceBetween size="m">
                <b>リソースがありません</b>
              </SpaceBetween>
            </Box>
          }
          filter={
            <TextFilter
              filteringPlaceholder="リソース検索"
              filteringText={filteringText}
              onChange={onChangeFilteringText}
            />
          }
          header={
            <Header
              actions={
                selectedResourceId === undefined ? undefined : (
                  <SpaceBetween direction="horizontal" size="xs">
                    <Button iconName={isOpen ? 'angle-left' : 'angle-right'} variant="icon" onClick={setToggleOpen} />
                  </SpaceBetween>
                )
              }
            >
              {selectedResourceId && <Button iconName="view-full" variant="icon" onClick={onClickExpand} />}
              リソース
            </Header>
          }
        />
      </div>
    );
  }

  return (
    <Container>
      <Button iconName="angle-right" variant="icon" onClick={setToggleOpen} />
    </Container>
  );
};
