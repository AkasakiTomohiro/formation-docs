import { isJsonSchemaPrimitiveType } from '../components/types/CloudFormationSchema';

import type {
  CloudFormationSchema,
  DefinedProperty,
  ReferenceProperty,
  ReferencePropertyWithDescription,
} from '../components/types/CloudFormationSchema';

export type ResourceTableItem = {
  id: string;
  property: string;
  type: string;
  description: string;
  value: string;
  children?: ResourceTableItem[];
};

/**
 * CloudFormationSchemaのと実際のテンプレートに定義されているプロパティをテーブルアイテムに変換する
 * @param schema CloudFormationSchema
 * @param actualProperties 実際のテンプレートに定義されているプロパティ
 * @returns テーブルアイテム
 */
export const createResourceTableItems = (
  schema: CloudFormationSchema,
  // biome-ignore lint/suspicious/noExplicitAny: <explanation>
  actualProperties: any,
): ResourceTableItem[] => {
  const items: ResourceTableItem[] = [];
  for (const [key, value] of Object.entries(schema.properties)) {
    if (schema.readOnlyProperties.includes(`/properties/${key}`)) {
      continue;
    }
    if ('$ref' in value) {
      const refItem = parseReferencePropertyToTableItem(
        '/properties',
        key,
        value,
        actualProperties,
        schema.definitions,
      );
      if (refItem) {
        items.push(refItem);
      }
    } else {
      const definedItem = parseDefinedPropertyToTableItem(
        '/properties',
        key,
        value,
        actualProperties,
        schema.definitions,
      );
      items.push(definedItem);
    }
  }
  return items;
};

/**
 * `CloudFormationSchema.properties`の$refを含まないプロパティをテーブルアイテムに変換する
 * @param parentId 親の`id`
 * @param propertyKey プロパティのキー名
 * @param property プロパティの定義
 * @param actualProperties 実際のテンプレートに定義されているプロパティ
 * @param definitions `CloudFormationSchema.definitions`
 * @return テーブルアイテム
 */
function parseDefinedPropertyToTableItem(
  parentId: string,
  propertyKey: string,
  property: DefinedProperty,
  // biome-ignore lint/suspicious/noExplicitAny: <explanation>
  actualProperties: any,
  definitions: CloudFormationSchema['definitions'] | undefined,
): ResourceTableItem {
  const result: ResourceTableItem = {
    id: `${parentId}/${propertyKey}`,
    property: propertyKey,
    type: covertType(property),
    description: property.description || '',
    value: covertValue(
      property,
      getActualProperties(propertyKey, actualProperties),
    ),
  };
  if (property.type === 'object' && property.properties) {
    result.children = [];
    for (const [definitionKey, definitionValue] of Object.entries(
      property.properties,
    )) {
      if ('$ref' in definitionValue) {
        const childItem = parseReferencePropertyToTableItem(
          `${parentId}/${propertyKey}`,
          definitionKey,
          definitionValue,
          getActualProperties(propertyKey, actualProperties),
          definitions,
        );
        if (childItem) {
          result.children.push(childItem);
        }
      } else {
        const childItem = parseDefinedPropertyToTableItem(
          `${parentId}/${propertyKey}`,
          definitionKey,
          definitionValue,
          getActualProperties(propertyKey, actualProperties),
          definitions,
        );
        result.children.push(childItem);
      }
    }
  }
  return result;
}

/**
 * `CloudFormationSchema.properties`の$refを含むプロパティをテーブルアイテムに変換する
 * @param parentId 親の`id`
 * @param propertyKey プロパティのキー名
 * @param property プロパティの定義
 * @param actualProperties 実際のテンプレートに定義されているプロパティ
 * @param definitions `CloudFormationSchema.definitions`
 * @return テーブルアイテム
 */
function parseReferencePropertyToTableItem(
  parentId: string,
  propertyKey: string,
  property: ReferenceProperty | ReferencePropertyWithDescription,
  // biome-ignore lint/suspicious/noExplicitAny: <explanation>
  actualProperties: any,
  definitions: CloudFormationSchema['definitions'] | undefined,
): ResourceTableItem | undefined {
  if (definitions === undefined) {
    return undefined;
  }

  const definitionKey = property.$ref.replace('#/definitions/', '');
  const definition = definitions[definitionKey];
  const result: ResourceTableItem = {
    id: `${parentId}/${propertyKey}`,
    property: propertyKey,
    type: covertType(definition),
    description: 'description' in property ? property.description || '' : '',
    value: covertValue(
      definition,
      getActualProperties(propertyKey, actualProperties),
    ),
  };
  if (definition.type === 'object') {
    result.children = [];
    for (const [definitionKey, definitionValue] of Object.entries(
      definition.properties,
    )) {
      if ('$ref' in definitionValue) {
        const childItem = parseReferencePropertyToTableItem(
          `${parentId}/${propertyKey}`,
          definitionKey,
          definitionValue,
          getActualProperties(propertyKey, actualProperties),
          definitions,
        );
        if (childItem) {
          result.children.push(childItem);
        }
        continue;
      }
      const childItem = parseDefinedPropertyToTableItem(
        `${parentId}/${propertyKey}`,
        definitionKey,
        definitionValue,
        getActualProperties(propertyKey, actualProperties),
        definitions,
      );
      result.children.push(childItem);
    }
  }

  return result;
}

function getActualProperties(
  propertyKey: string,
  // biome-ignore lint/suspicious/noExplicitAny: <explanation>
  actualProperties: any,
  // biome-ignore lint/suspicious/noExplicitAny: <explanation>
): any {
  if (actualProperties === undefined || actualProperties === null) {
    return {};
  }
  return actualProperties[propertyKey];
}

function isIntrinsicFunction(
  // biome-ignore lint/suspicious/noExplicitAny: <explanation>
  actualProperty: any,
): boolean {
  const intrinsicFunctions = [
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
  ];
  const objKeys = Object.keys(actualProperty);
  return intrinsicFunctions.some((intrinsicFunction) => {
    return objKeys.includes(intrinsicFunction);
  });
}

function covertValue(
  property: DefinedProperty,
  // biome-ignore lint/suspicious/noExplicitAny: <explanation>
  actualProperty: any,
): string {
  if (actualProperty === undefined || actualProperty === null) {
    return '';
  }
  if (isIntrinsicFunction(actualProperty)) {
    return '';
  }
  if (isJsonSchemaPrimitiveType(property.type)) {
    return actualProperty;
  }
  return '';
}

/**
 * `property.type`を型の文字列に変換する
 * @param property `$ref`を含まないプロパティ
 * @returns 型の文字列
 */
function covertType(property: DefinedProperty): string {
  if (Array.isArray(property.type)) {
    return property.type.join(' | ');
  }
  if (property.type === 'string') {
    if (property.enum) {
      return property.enum.map((item) => item.toString()).join(' | ');
    }
  }
  if (property.type === 'array') {
    if (property.items && !('$ref' in property.items)) {
      if (isJsonSchemaPrimitiveType(property.items.type)) {
        const type = covertType(property.items);
        if (type.includes(' | ')) {
          return `( ${type} )[]`;
        }
        return `${type}[]`;
      }
    }
  }
  return property.type;
}
