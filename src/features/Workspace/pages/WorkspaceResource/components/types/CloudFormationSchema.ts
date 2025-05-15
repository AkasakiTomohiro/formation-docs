export type JsonSchemaType =
  | 'array'
  | 'boolean'
  | 'integer'
  | 'null'
  | 'number'
  | 'object'
  | 'string'
  | (string & {});

export function isJsonSchemaPrimitiveType(
  type: JsonSchemaType | JsonSchemaType[],
): boolean {
  if (Array.isArray(type)) {
    return false;
  }
  return (
    type === 'boolean' ||
    type === 'integer' ||
    type === 'number' ||
    type === 'string'
  );
}

export type ReplacementStrategy = 'create_then_delete' | 'delete_then_create';

export interface Tagging {
  taggable: boolean;
  tagOnCreate?: boolean;
  tagUpdatable?: boolean;
  cloudFormationSystemTags?: boolean;
  permissions?: string[];
  tagProperty?: string; // assuming JSON Pointer is a string
}

export type PropertyPath = `/properties/${string}`;

export type ReferenceProperty = {
  $ref: `#/definitions/${string}`;
};
export type ReferencePropertyWithDescription = ReferenceProperty & {
  description: string;
};

export type DefinedProperty = {
  type: JsonSchemaType | JsonSchemaType[];
  description?: string;
  insertionOrder?: boolean;
  arrayType?: 'Standard' | 'AttributeList';
  relationshipRef?: {
    typeName?: string;
    propertyPath?: PropertyPath;
    publisherId?: string;
    majorVersion?: number;
  };
  maximum?: number;
  minimum?: number;
  maxLength?: number;
  minLength?: number;
  pattern?: string;
  items?: Property;
  properties?: Record<string, Property>;
  enum?: string[];
};

export type Property =
  | ReferenceProperty
  | ReferencePropertyWithDescription
  | DefinedProperty;

export interface Handler {
  permissions: string[];
  timeoutInMinutes: number;
}

export interface Handlers {
  create: Handler;
  read: Handler;
  update: Handler;
  delete: Handler;
  list: Handler;
}

export interface ResourceLink {
  templateUri: string;
  mappings: string; // assuming JSON Pointer is a string
}

export interface Definition {
  type: JsonSchemaType;
  additionalProperties: boolean;
  properties: Record<string, Property>;
  required: string[];
  description: string;
}

export interface CloudFormationSchema {
  typeName: string;
  description: string;
  sourceUrl: string;
  documentationUrl: string;
  taggable: boolean;
  tagging: Tagging;
  replacementStrategy: ReplacementStrategy;
  properties: Record<string, Property>;

  additionalIdentifiers: PropertyPath[];
  definitions?: Record<string, Definition>;
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
  // biome-ignore lint/suspicious/noExplicitAny: <explanation>
  typeConfiguration?: Record<string, any>;
  resourceLink?: ResourceLink;
}

export const intrinsicFunctions = [
  'Fn::Base64',
  'Fn::Cidr',
  'Fn::And',
  'Fn::Equals',
  'Fn::If',
  'Fn::Not',
  'Fn::Or',
  'Fn::FindInMap',
  'Fn::ForEach',
  'Fn::GetAtt',
  'Fn::GetAZs',
  'Fn::ImportValue',
  'Fn::Join',
  'Fn::Length',
  'Fn::Select',
  'Fn::Split',
  'Fn::Sub',
  'Fn::ToJsonString',
  'Fn::Transform',
  'Ref',
] as const;
export type IntrinsicFunction = (typeof intrinsicFunctions)[number];

export const pseudoProperties = [
  'AWS::AccountId',
  'AWS::NotificationARNs',
  'AWS::NoValue',
  'AWS::Partition',
  'AWS::Region',
  'AWS::StackId',
  'AWS::StackName',
  'AWS::URLSuffix',
] as const;
export type PseudoProperty = (typeof pseudoProperties)[number];
