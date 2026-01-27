import { Input, TokenGroup } from '@cloudscape-design/components';
import type { InputProps, TokenGroupProps } from '@cloudscape-design/components';

export type SearchFormPresentationProps = {
  /**
   * 検索ワード
   */
  searchValue: string;

  /**
   * 検索フィールドのonChangeハンドラー
   */
  onChangeInput: InputProps['onChange'];

  /**
   * 検索フィールドのonKeyDownハンドラー
   */
  onKeyDownInput: InputProps['onKeyDown'];

  /**
   * トークングループ
   */
  tokenGroup: TokenGroupProps.Item[];

  /**
   * トークングループのonDismissハンドラー
   */
  onDismissToken: TokenGroupProps['onDismiss'];
};

export const SearchFormPresentation = (props: SearchFormPresentationProps): JSX.Element => {
  return (
    <>
      <Input
        type="search"
        value={props.searchValue}
        placeholder="リソース検索"
        ariaLabel="リソース検索"
        onChange={props.onChangeInput}
        onKeyDown={props.onKeyDownInput}
      />
      <TokenGroup onDismiss={props.onDismissToken} items={props.tokenGroup} />
    </>
  );
};
