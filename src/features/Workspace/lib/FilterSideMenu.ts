import type { SideNavigationProps } from '@cloudscape-design/components';
import type { TemplateSummary } from '../../../invoke/Stack';

type SectionGroupItem =
  | SideNavigationProps.Section
  | SideNavigationProps.Link
  | SideNavigationProps.LinkGroup
  | SideNavigationProps.ExpandableLinkGroup;

export const filterSideMenu = (
  sideMenu: TemplateSummary[],
  searchValues: string[],
): SideNavigationProps.Item[] => {
  const items: SideNavigationProps.Item[] = [];

  for (const item of sideMenu.sort((a, b) =>
    a.stackName.localeCompare(b.stackName),
  )) {
    const newChildren: SectionGroupItem[] = [
      {
        type: 'link',
        text: 'Overview',
        href: `${item.id}/${item.stackName}`,
      },
    ];

    // スタック名に含まれている検索文字を取り除く
    const remainingSearchValuesStack = searchValues.filter(
      (searchValue) =>
        item.stackName.toLowerCase().includes(searchValue.toLowerCase()) ===
        false,
    );

    for (const resource of item.resources.sort((a, b) =>
      a.serviceName.localeCompare(b.serviceName),
    )) {
      // サービス名に含まれている検索文字を取り除く
      const remainingSearchValues = remainingSearchValuesStack.filter(
        (searchValue) =>
          resource.serviceName
            .toLowerCase()
            .includes(searchValue.toLowerCase()) === false,
      );

      if (remainingSearchValues.length === 0) {
        // スタック名もしくはサービス名に検索文字が含まれている場合は、すべてのリソースを表示する
        newChildren.push({
          type: 'section',
          text: resource.serviceName,
          items: resource.recourseType
            .sort((a, b) => a.localeCompare(b))
            .map((rType) => ({
              type: 'link',
              text: rType,
              href: `${item.id}/${item.stackName}/${resource.serviceName}/${rType}`,
            })),
        });
      } else {
        // スタック名もしくはサービス名に検索文字が含まれていない場合は、リソース名に検索文字が含まれているものを表示する
        const newResource: SideNavigationProps.Section = {
          type: 'section',
          text: resource.serviceName,
          items: resource.recourseType
            .sort((a, b) => a.localeCompare(b))
            .filter((rType) =>
              remainingSearchValues.every((searchValue) =>
                rType.toLowerCase().includes(searchValue.toLowerCase()),
              ),
            )
            .map((rType) => ({
              type: 'link',
              text: rType,
              href: `${item.id}/${item.stackName}/${resource.serviceName}/${rType}`,
            })),
        };
        if (newResource.items.length > 0) {
          newChildren.push(newResource);
        }
      }
    }
    if (newChildren.length === 0) {
      continue;
    }
    const newItem: SideNavigationProps.SectionGroup = {
      type: 'section-group',
      title: item.stackName,
      items: newChildren,
    };
    items.push(newItem);
    items.push({ type: 'divider' });
  }

  return items;
};
