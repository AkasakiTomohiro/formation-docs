import { useWorkspaceResourceContext } from '../../../../contexts';
import { SearchFormPresentation } from './SearchForm.presentation';

export const SearchForm = (): JSX.Element => {
  const { searchValue, setSearchValue, tokenGroup, setTokenGroup } = useWorkspaceResourceContext();

  return (
    <SearchFormPresentation
      searchValue={searchValue}
      onChangeInput={({ detail }) => setSearchValue(detail.value)}
      onKeyDownInput={({ detail }) => {
        if (detail.key === 'Enter') {
          if (searchValue === '') {
            return;
          }
          setTokenGroup((prev) => [...prev, { label: searchValue }]);
          setSearchValue('');
        }
      }}
      tokenGroup={tokenGroup}
      onDismissToken={({ detail: { itemIndex } }) => {
        setTokenGroup([...tokenGroup.slice(0, itemIndex), ...tokenGroup.slice(itemIndex + 1)]);
      }}
    />
  );
};
