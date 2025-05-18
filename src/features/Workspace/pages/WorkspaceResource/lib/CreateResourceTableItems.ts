import {
  intrinsicFunctions,
  isJsonSchemaPrimitiveType,
  pseudoProperties,
} from '../components/types/CloudFormationSchema';

import type {
  CloudFormationSchema,
  DefinedProperty,
  IntrinsicFunction,
  PseudoProperty,
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
  if (
    definitions !== undefined &&
    property.type === 'array' &&
    property.items !== undefined &&
    '$ref' in property.items
  ) {
    const definitionKey = property.items.$ref.replace('#/definitions/', '');
    const definition = definitions[definitionKey];
    const propertyList =
      getActualProperties(propertyKey, actualProperties) ?? [];
    result.children = [];
    for (const [index, property] of Object.entries(propertyList)) {
      const childItem: ResourceTableItem = {
        id: `${parentId}/${propertyKey}/${index}`,
        property: index,
        type: 'object',
        description: '',
        value: '',
      };
      childItem.children = [];
      for (const [definitionKey, definitionValue] of Object.entries(
        definition.properties,
      )) {
        if ('$ref' in definitionValue) {
          const grandChildItem = parseReferencePropertyToTableItem(
            `${parentId}/${propertyKey}`,
            definitionKey,
            definitionValue,
            property,
            definitions,
          );
          if (grandChildItem) {
            childItem.children.push(grandChildItem);
          }
          continue;
        }
        const grandChildItem = parseDefinedPropertyToTableItem(
          `${parentId}/${propertyKey}/${index}`,
          definitionKey,
          definitionValue,
          property,
          definitions,
        );
        childItem.children.push(grandChildItem);
      }
      result.children.push(childItem);
    }
  }
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
  if (result.children !== undefined) {
    if (result.children.length === 0) {
      // biome-ignore lint/performance/noDelete: <explanation>
      delete result.children;
    } else {
      result.value = '';
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
  if (result.children !== undefined) {
    if (result.children.length === 0) {
      // biome-ignore lint/performance/noDelete: <explanation>
      delete result.children;
    } else {
      result.value = '';
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
  if (typeof actualProperties === 'object' && propertyKey in actualProperties) {
    return actualProperties[propertyKey];
  }
  return undefined;
}

function isPseudoProperty(property: string): PseudoProperty | undefined {
  for (const pseudo of pseudoProperties) {
    if (property === pseudo) {
      return pseudo as PseudoProperty;
    }
  }
  return undefined;
}

function isIntrinsicFunction(
  // biome-ignore lint/suspicious/noExplicitAny: <explanation>
  actualProperty: any,
): IntrinsicFunction | undefined {
  const objKeys = Object.keys(actualProperty);
  for (const intrinsic of intrinsicFunctions) {
    if (objKeys.includes(intrinsic)) {
      return intrinsic as IntrinsicFunction;
    }
  }
  return undefined;
}

function convertIntrinsicFunctionValue(
  intrinsic: IntrinsicFunction,
  // biome-ignore lint/suspicious/noExplicitAny: <explanation>
  actualProperty: any,
): string {
  switch (intrinsic) {
    case 'Fn::Base64': {
      return JSON.stringify(actualProperty, undefined, 2);
    }
    case 'Fn::Cidr': {
      return JSON.stringify(actualProperty, undefined, 2);
    }
    case 'Fn::And': {
      return JSON.stringify(actualProperty, undefined, 2);
    }
    case 'Fn::Equals': {
      return JSON.stringify(actualProperty, undefined, 2);
    }
    case 'Fn::If': {
      return JSON.stringify(actualProperty, undefined, 2);
    }
    case 'Fn::Not': {
      return JSON.stringify(actualProperty, undefined, 2);
    }
    case 'Fn::Or': {
      return JSON.stringify(actualProperty, undefined, 2);
    }
    case 'Fn::FindInMap': {
      return JSON.stringify(actualProperty, undefined, 2);
    }
    case 'Fn::ForEach': {
      return JSON.stringify(actualProperty, undefined, 2);
    }
    case 'Fn::GetAtt': {
      return JSON.stringify(actualProperty, undefined, 2);
    }
    case 'Fn::GetAZs': {
      return JSON.stringify(actualProperty, undefined, 2);
    }
    case 'Fn::ImportValue': {
      return JSON.stringify(actualProperty, undefined, 2);
    }
    case 'Fn::Join': {
      const delimiter = actualProperty['Fn::Join'][0];
      // biome-ignore lint/suspicious/noExplicitAny: <explanation>
      const pieces = actualProperty['Fn::Join'][1].map((value: any) => {
        const intrinsic = isIntrinsicFunction(value);
        if (intrinsic !== undefined) {
          return convertIntrinsicFunctionValue(intrinsic, value);
        }
        return value;
      });
      return pieces.join(delimiter);
    }
    case 'Fn::Length': {
      return JSON.stringify(actualProperty, undefined, 2);
    }
    case 'Fn::Select': {
      return JSON.stringify(actualProperty, undefined, 2);
    }
    case 'Fn::Split': {
      return JSON.stringify(actualProperty, undefined, 2);
    }
    case 'Fn::Sub': {
      return JSON.stringify(actualProperty, undefined, 2);
    }
    case 'Fn::ToJsonString': {
      return JSON.stringify(actualProperty, undefined, 2);
    }
    case 'Fn::Transform': {
      return JSON.stringify(actualProperty, undefined, 2);
    }
    case 'Ref': {
      let value = actualProperty.Ref;
      const intrinsic = isIntrinsicFunction(value);
      if (intrinsic !== undefined) {
        value = convertIntrinsicFunctionValue(intrinsic, value);
      }
      const pseudo = isPseudoProperty(value);
      if (pseudo !== undefined) {
        // 疑似パラメータ：<疑似パラメータ>
        return `<${pseudo}>`;
      }
      // 疑似パラメータ以外：<Ref: 論理ID or Parameter>
      return `<Ref: ${value}>`;
    }
  }
}

function covertValue(
  property: DefinedProperty,
  // biome-ignore lint/suspicious/noExplicitAny: <explanation>
  actualProperty: any,
): string {
  if (actualProperty === undefined) {
    return '';
  }
  const intrinsic = isIntrinsicFunction(actualProperty);
  if (intrinsic !== undefined) {
    return convertIntrinsicFunctionValue(intrinsic, actualProperty);
  }
  if (isJsonSchemaPrimitiveType(property.type)) {
    return actualProperty;
  }
  if (Array.isArray(actualProperty)) {
    if (actualProperty.length !== 0) {
      const firstItemType = typeof actualProperty[0];
      if (firstItemType !== 'object' && firstItemType !== 'function') {
        return `[ ${actualProperty.join(', ')} ]`;
      }
    }
  }

  // これから以外の場合はJSON.stringifyで文字列化する
  return JSON.stringify(actualProperty, undefined, 2);
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
