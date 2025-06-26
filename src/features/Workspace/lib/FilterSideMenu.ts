import type { SideNavigationProps } from '@cloudscape-design/components';
import type { TemplateSummary } from '../../../invoke/Stack';

type SectionGroupItem =
  | SideNavigationProps.Section
  | SideNavigationProps.Link
  | SideNavigationProps.LinkGroup
  | SideNavigationProps.ExpandableLinkGroup;

export const ManualManagementId = 'manualManagement';

export const filterSideMenu = (
  sideMenu: TemplateSummary[],
  searchValues: string[],
): SideNavigationProps.Item[] => {
  const items: SideNavigationProps.Item[] = [];

  for (const item of sideMenu.sort((a, b) =>
    a.sectionGroupName.localeCompare(b.sectionGroupName),
  )) {
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
      (searchValue) =>
        item.sectionGroupName
          .toLowerCase()
          .includes(searchValue.toLowerCase()) === false,
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
              href:
                item.id === ManualManagementId
                  ? hrefBuilder({
                      type: 'manual',
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
              remainingSearchValues.every((searchValue) =>
                rType.toLowerCase().includes(searchValue.toLowerCase()),
              ),
            )
            .map((rType) => ({
              type: 'link',
              text: rType,
              href:
                item.id === ManualManagementId
                  ? hrefBuilder({
                      type: 'manual',
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
    if (newChildren.length === 0) {
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

export type HrefType =
  | {
      type: 'overview';
      stackId: string;
      sectionGroupName: string;
    }
  | {
      type: 'resource';
      stackId: string;
      sectionGroupName: string;
      serviceName: string;
      resourceType: string;
    }
  | {
      type: 'manualOverview';
    }
  | {
      type: 'manual';
      serviceName: string;
      resourceType: string;
    };

const hrefBuilder = (props: HrefType): string => {
  switch (props.type) {
    case 'overview':
      return `overview:${props.stackId}/${props.sectionGroupName}`;
    case 'resource':
      return `resource:${props.stackId}/${props.sectionGroupName}/${props.serviceName}/${props.resourceType}`;
    case 'manualOverview':
      return 'manualOverview:';
    case 'manual':
      return `manual:${props.serviceName}/${props.resourceType}`;
  }
};

export const hrefParser = (href: string): HrefType => {
  // hrefを:より前と:より後ろに分割
  const [type, rest] = href.split(':');

  switch (type) {
    case 'overview': {
      // hrefの:より後ろの部分を/で分割
      const [stackId, sectionGroupName] = rest.split('/');
      return {
        type: 'overview',
        stackId: stackId,
        sectionGroupName: sectionGroupName,
      };
    }
    case 'resource': {
      // hrefの:より後ろの部分を/で分割
      const [stackId, sectionGroupName, serviceName, resourceType] =
        rest.split('/');
      return {
        type: 'resource',
        stackId: stackId,
        sectionGroupName: sectionGroupName,
        serviceName: serviceName,
        resourceType: resourceType,
      };
    }
    case 'manualOverview': {
      return { type: 'manualOverview' };
    }
    case 'manual': {
      // hrefの:より後ろの部分を/で分割
      const [serviceName, resourceType] = rest.split('/');
      return {
        type: 'manual',
        serviceName: serviceName,
        resourceType: resourceType,
      };
    }
    default:
      throw new Error(`Unknown href type: ${type}`);
  }
};
