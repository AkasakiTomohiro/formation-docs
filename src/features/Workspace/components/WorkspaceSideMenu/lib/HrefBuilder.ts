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
      type: 'manualResource';
      serviceName: string;
      resourceType: string;
    };

export const hrefBuilder = (props: HrefType): string => {
  switch (props.type) {
    case 'overview':
      return `overview:${props.stackId}/${props.sectionGroupName}`;
    case 'resource':
      return `resource:${props.stackId}/${props.sectionGroupName}/${props.serviceName}/${props.resourceType}`;
    case 'manualOverview':
      return 'manualOverview:';
    case 'manualResource':
      return `manualResource:${props.serviceName}/${props.resourceType}`;
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
      const [stackId, sectionGroupName, serviceName, resourceType] = rest.split('/');
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
    case 'manualResource': {
      // hrefの:より後ろの部分を/で分割
      const [serviceName, resourceType] = rest.split('/');
      return {
        type: 'manualResource',
        serviceName: serviceName,
        resourceType: resourceType,
      };
    }
    default:
      throw new Error(`Unknown href type: ${type}`);
  }
};
