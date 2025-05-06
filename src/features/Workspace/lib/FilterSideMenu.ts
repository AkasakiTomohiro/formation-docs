import type { SideNavigationProps } from '@cloudscape-design/components';
import type { TemplateSummary } from '../../../invoke/Stack';

type SectionGroupItem =
  | SideNavigationProps.Section
  | SideNavigationProps.Link
  | SideNavigationProps.LinkGroup
  | SideNavigationProps.ExpandableLinkGroup;

export const filterSideMenu = (
  sideMenu: TemplateSummary[],
  searchValue: string,
): SideNavigationProps.Item[] => {
  const items: SideNavigationProps.Item[] = [];

  for (const item of sideMenu.sort((a, b) =>
    a.stackName.localeCompare(b.stackName),
  )) {
    const newChildren: SectionGroupItem[] = [
      {
        type: 'link',
        text: 'Detail',
        href: `${item.id}/${item.stackName}`,
      },
    ];
    if (item.stackName.toLowerCase().includes(searchValue.toLowerCase())) {
      // スタック名に検索文字が含まれている場合はすべてのリソースを表示する
      for (const resource of item.resources.sort((a, b) =>
        a.serviceName.localeCompare(b.serviceName),
      )) {
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
      }
    } else {
      // スタック名に検索文字が含まれていない場合は、サービス名もしくはリソース名に検索文字が含まれているものを表示する
      for (const resource of item.resources.sort((a, b) =>
        a.serviceName.localeCompare(b.serviceName),
      )) {
        if (
          resource.serviceName.toLowerCase().includes(searchValue.toLowerCase())
        ) {
          // サービス名に検索文字が含まれている場合は、すべてのリソースを表示する
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
          // サービス名に検索文字が含まれていない場合は、リソース名に検索文字が含まれているものを表示する
          const newResource: SideNavigationProps.Section = {
            type: 'section',
            text: resource.serviceName,
            items: resource.recourseType
              .sort((a, b) => a.localeCompare(b))
              .filter((rType) =>
                rType.toLowerCase().includes(searchValue.toLowerCase()),
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
