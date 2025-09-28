import { ManualManagementId } from '../../../contexts';
import { hrefBuilder } from './HrefBuilder';
import type { SideNavigationProps } from '@cloudscape-design/components';
import type { TemplateSummary } from '../../../contexts/lib/LoadTemplateSummary';

type SectionGroupItem =
  | SideNavigationProps.Section
  | SideNavigationProps.Link
  | SideNavigationProps.LinkGroup
  | SideNavigationProps.ExpandableLinkGroup;

export const filterSideMenu = (sideMenu: TemplateSummary[], searchValues: string[]): SideNavigationProps.Item[] => {
  const items: SideNavigationProps.Item[] = [];

  for (const item of sideMenu.sort((a, b) => a.sectionGroupName.localeCompare(b.sectionGroupName))) {
    const newChildren: SectionGroupItem[] = [
      {
        type: 'link',
        text: 'Overview',
        href:
          item.id === ManualManagementId
            ? hrefBuilder({
                type: 'manualOverview',
              })
            : hrefBuilder({
                type: 'overview',
                stackId: item.id,
                sectionGroupName: item.sectionGroupName,
              }),
      },
    ];

    // スタック名に含まれている検索文字を取り除く
    const remainingSearchValuesStack = searchValues.filter(
      (searchValue) => item.sectionGroupName.toLowerCase().includes(searchValue.toLowerCase()) === false,
    );

    for (const resource of item.resources.sort((a, b) => a.serviceName.localeCompare(b.serviceName))) {
      // サービス名に含まれている検索文字を取り除く
      const remainingSearchValues = remainingSearchValuesStack.filter(
        (searchValue) => resource.serviceName.toLowerCase().includes(searchValue.toLowerCase()) === false,
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
              href:
                item.id === ManualManagementId
                  ? hrefBuilder({
                      type: 'manualResource',
                      serviceName: resource.serviceName,
                      resourceType: rType,
                    })
                  : hrefBuilder({
                      type: 'resource',
                      stackId: item.id,
                      sectionGroupName: item.sectionGroupName,
                      serviceName: resource.serviceName,
                      resourceType: rType,
                    }),
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
              remainingSearchValues.every((searchValue) => rType.toLowerCase().includes(searchValue.toLowerCase())),
            )
            .map((rType) => ({
              type: 'link',
              text: rType,
              href:
                item.id === ManualManagementId
                  ? hrefBuilder({
                      type: 'manualResource',
                      serviceName: resource.serviceName,
                      resourceType: rType,
                    })
                  : hrefBuilder({
                      type: 'resource',
                      stackId: item.id,
                      sectionGroupName: item.sectionGroupName,
                      serviceName: resource.serviceName,
                      resourceType: rType,
                    }),
            })),
        };

        if (newResource.items.length > 0) {
          newChildren.push(newResource);
        }
      }
    }
    // overviewの要素が初期に追加されているため、検索結果が1件もない場合配列の要素は1つ
    if (newChildren.length <= 1 && item.id !== ManualManagementId) {
      continue;
    }
    const newItem: SideNavigationProps.SectionGroup = {
      type: 'section-group',
      title: item.sectionGroupName,
      items: newChildren,
    };
    items.push(newItem);
    items.push({ type: 'divider' });
  }

  return items;
};
