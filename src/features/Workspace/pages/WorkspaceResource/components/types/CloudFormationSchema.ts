type ReplacementStrategy = 'create_then_delete' | 'delete_then_create';

interface Tagging {
  taggable: boolean;
  tagOnCreate: boolean;
  tagUpdatable: boolean;
  cloudFormationSystemTags: boolean;
  tagProperty: string; // assuming JSON Pointer is a string
}

type ReferenceProperty = {
  $ref: `#/definitions/${string}`;
};
type ReferencePropertyWithDescription = ReferenceProperty & {
  description: string;
};

type Property =
  | {
      description: string;
      type: string | string[];
      items?:
        | ReferenceProperty
        | {
            type: string;
            enum: string[];
          };
      enum?: string[];
    }
  | ReferencePropertyWithDescription;

interface Handler {
  permissions: string[];
  timeoutInMinutes: number;
}

interface Handlers {
  create: Handler;
  read: Handler;
  update: Handler;
  delete: Handler;
  list: Handler;
}

type PropertyPath = `/properties/${string}`;

interface ResourceLink {
  templateUri: string;
  mappings: string; // assuming JSON Pointer is a string
}

interface Definition {
  type: 'object';
  additionalProperties: boolean;
  properties: Record<string, Property>;
  required: string[];
  description: string;
}

interface CloudFormationSchema {
  typeName: string;
  description: string;
  sourceUrl: string;
  documentationUrl: string;
  replacementStrategy: ReplacementStrategy;
  taggable: boolean;
  tagging: Tagging;
  definitions?: Record<string, Definition>;
  properties: Record<string, Property>;
  required: string[];
  propertyTransform: Record<string, string>;
  handlers: Handlers;
  readOnlyProperties: PropertyPath[];
  writeOnlyProperties: PropertyPath[];
  conditionalCreateOnlyProperties: PropertyPath[];
  nonPublicProperties: PropertyPath[];
  nonPublicDefinitions: PropertyPath[];
  createOnlyProperties: PropertyPath[];
  deprecatedProperties: PropertyPath[];
  primaryIdentifier: PropertyPath[];
  additionalIdentifiers: PropertyPath[];
  // biome-ignore lint/suspicious/noExplicitAny: <explanation>
  typeConfiguration?: Record<string, any>;
  resourceLink?: ResourceLink;
}
