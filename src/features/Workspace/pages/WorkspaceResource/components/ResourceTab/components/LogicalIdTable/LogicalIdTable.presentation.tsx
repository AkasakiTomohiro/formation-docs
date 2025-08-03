import { Box, Button, Container, Header, Link, SpaceBetween, Table, TextFilter } from '@cloudscape-design/components';

import type { ButtonProps, LinkProps, TextFilterProps } from '@cloudscape-design/components';

export type LogicalIdTablePresentationProps = {
  /**
   * フィルタリングテキスト
   */
  filteringText: string;

  /**
   * リソースのリスト
   */
  resourceList: string[];

  /**
   * 論理IDテーブルを開いているか
   */
  isOpen: boolean;

  /**
   * 選択している論理ID
   */
  selectedLogicalId?: string;

  /**
   * 論理IDクリック時のイベントハンドラ
   */
  onCliCkLogicalId: (item: string) => LinkProps['onClick'];

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
};

export const LogicalIdTablePresentation = ({
  filteringText,
  resourceList,
  isOpen,
  selectedLogicalId,
  onCliCkLogicalId,
  onChangeFilteringText,
  setToggleOpen,
  onClickExpand,
  isLoading,
}: LogicalIdTablePresentationProps): JSX.Element => {
  if (isOpen) {
    return (
      <div
        style={{
          width: selectedLogicalId === undefined ? '100%' : '300px',
          minWidth: '300px',
          height: '100%',
        }}
      >
        <Table
          columnDefinitions={[
            {
              id: 'logicalId',
              header: 'logical id',
              cell: (item: string) => (
                <Link href="#" onClick={onCliCkLogicalId(item)}>
                  {item}
                </Link>
              ),
              sortingField: 'name',
              isRowHeader: true,
            },
          ]}
          enableKeyboardNavigation
          items={resourceList}
          loadingText="Loading resources"
          loading={isLoading}
          sortingDisabled
          empty={
            <Box margin={{ vertical: 'xs' }} textAlign="center" color="inherit">
              <SpaceBetween size="m">
                <b>No resources</b>
              </SpaceBetween>
            </Box>
          }
          filter={
            <TextFilter
              filteringPlaceholder="Search Resource"
              filteringText={filteringText}
              onChange={onChangeFilteringText}
            />
          }
          header={
            <Header
              actions={
                selectedLogicalId === undefined ? undefined : (
                  <SpaceBetween direction="horizontal" size="xs">
                    <Button iconName={isOpen ? 'angle-left' : 'angle-right'} variant="icon" onClick={setToggleOpen} />
                  </SpaceBetween>
                )
              }
            >
              {selectedLogicalId && <Button iconName="view-full" variant="icon" onClick={onClickExpand} />}
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
