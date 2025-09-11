import { hrefBuilder } from '../../../components';
import {
  intrinsicFunctions,
  isJsonSchemaPrimitiveType,
  pseudoProperties,
} from '../components/types/CloudFormationSchema';

import type { OverviewTabAttr, ResourceTabAttr } from '../../../contexts';
import type {
  CloudFormationSchema,
  DefinedProperty,
  IntrinsicFunction,
  Property,
  ReferenceProperty,
  ReferencePropertyWithDescription,
} from '../components/types/CloudFormationSchema';

export type ResourceTableItem = {
  id: string;
  property: string;
  type: string;
  description: string;
  value: ResourceTableItemValue | undefined;
  children?: ResourceTableItem[];
};

export type ResourceTableItemValue =
  | {
      type: 'value';
      value: string;
    }
  | (OverviewTabAttr & {
      type: 'overview';
      value: string;
    })
  | (ResourceTabAttr & {
      type: 'resource';
      value: string;
    })
  | {
      type: 'array';
      value: Extract<ResourceTableItemValue, { type: 'resource' | 'overview' | 'value' }>[];
    };

export type CreateResourceTableItemsOption = {
  stackId: string;
  stackName: string;
  parameters: string[];
  resources: Record<
    string,
    {
      serviceName: string;
      recourseType: string;
    }
  >;
  externalResources: Record<string, { stackId: string; stackName: string }>;
};

const DefaultOption: CreateResourceTableItemsOption = {
  parameters: [],
  resources: {},
  externalResources: {},
  stackId: '',
  stackName: '',
};

/**
 * CloudFormationSchemaのと実際のテンプレートに定義されているプロパティをテーブルアイテムに変換する
 * @param schema CloudFormationSchema
 * @param actualProperties 実際のテンプレートに定義されているプロパティ
 * @param options 組み込み関数用のデータ（ParameterとResourceList）
 * @returns テーブルアイテム
 */
export const createResourceTableItems = (
  schema: CloudFormationSchema,
  // biome-ignore lint/suspicious/noExplicitAny: <explanation>
  actualProperties: any,
  options: CreateResourceTableItemsOption = DefaultOption,
): ResourceTableItem[] => {
  const items: ResourceTableItem[] = [];
  for (const [key, value] of Object.entries(schema.properties)) {
    if (schema.readOnlyProperties.includes(`/Properties/${key}`)) {
      continue;
    }
    if ('$ref' in value) {
      const refItem = parseReferencePropertyToTableItem(
        '/Properties',
        key,
        value,
        actualProperties,
        schema.definitions,
        options,
      );
      if (refItem) {
        items.push(refItem);
      }
    } else {
      const definedItem = parseDefinedPropertyToTableItem(
        '/Properties',
        key,
        value,
        actualProperties,
        schema.definitions,
        options,
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
 * @param options 組み込み関数用のデータ（ParameterとResourceList）
 * @return テーブルアイテム
 */
function parseDefinedPropertyToTableItem(
  parentId: string,
  propertyKey: string,
  property: DefinedProperty,
  // biome-ignore lint/suspicious/noExplicitAny: <explanation>
  actualProperties: any,
  definitions: CloudFormationSchema['definitions'] | undefined,
  options: CreateResourceTableItemsOption,
): ResourceTableItem {
  const convertedValue = convertValue(property, getActualProperties(propertyKey, actualProperties), options);
  const result: ResourceTableItem = {
    id: `${parentId}/${propertyKey}`,
    property: propertyKey,
    type: convertType(property),
    description: property.description || '',
    value: convertedValue,
  };

  // プロパティが配列かつ、itemsが定義されている場合は、子要素を取得する
  if (
    definitions !== undefined &&
    property.type === 'array' &&
    property.items !== undefined &&
    '$ref' in property.items
  ) {
    const definitionKey = property.items.$ref.replace('#/definitions/', '');
    const definition = definitions[definitionKey];
    const propertyList = getActualProperties(propertyKey, actualProperties) ?? [];
    result.children = [];

    // 配列のIndex番号ごとに子要素を作成する
    for (const [index, property] of Object.entries(propertyList)) {
      const childItem: ResourceTableItem = {
        id: `${result.id}/${index}`,
        property: index,
        type: 'object',
        description: '',
        value: {
          type: 'value',
          value: '',
        },
      };
      childItem.children = parseChildrenPropertyToTableItem(
        definition.properties,
        childItem.id,
        property,
        definitions,
        options,
      );
      result.children.push(childItem);
    }
    if (result.children.length === 0) {
      result.children = undefined;
    }
  }

  // プロパティがオブジェクトかつpropertiesが定義されている場合は、子要素を取得する
  if (property.type === 'object' && property.properties) {
    result.children = parseChildrenPropertyToTableItem(
      property.properties,
      result.id,
      getActualProperties(propertyKey, actualProperties),
      definitions,
      options,
    );
  }

  // 子要素がある場合はvalueを空文字にする
  if (result.children !== undefined) {
    if (result.children.length !== 0) {
      result.value = {
        type: 'value',
        value: '',
      };
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
  options: CreateResourceTableItemsOption,
): ResourceTableItem | undefined {
  // ReferenceProperty場合はdefinitionsがundefinedになりえないため。処理を中断する
  if (definitions === undefined) {
    return undefined;
  }

  const definitionKey = property.$ref.replace('#/definitions/', '');
  const definition = definitions[definitionKey];
  const convertedValue = convertValue(definition, getActualProperties(propertyKey, actualProperties), options);
  const result: ResourceTableItem = {
    id: `${parentId}/${propertyKey}`,
    property: propertyKey,
    type: convertType(definition),
    description: 'description' in property ? property.description || '' : '',
    value: convertedValue,
  };

  // プロパティがオブジェクトの場合は、子要素を取得する
  if (definition.type === 'object') {
    result.children = parseChildrenPropertyToTableItem(
      definition.properties,
      result.id,
      getActualProperties(propertyKey, actualProperties),
      definitions,
      options,
    );
  }

  // 子要素がある場合はvalueを空文字にする
  if (result.children !== undefined) {
    if (result.children.length !== 0) {
      result.value = {
        type: 'value',
        value: '',
      };
    }
  }

  return result;
}

/**
 * `CloudFormationSchema.properties`の子要素をテーブルアイテムに変換する
 * @param childrenProperties 子要素のプロパティ
 * @param parentId 親の`id`
 * @param actualProperties 実際のテンプレートに定義されているプロパティ
 * @param definitions `CloudFormationSchema.definitions`
 * @return テーブルアイテム
 */
function parseChildrenPropertyToTableItem(
  childrenProperties: Record<string, Property>,
  parentId: string,
  // biome-ignore lint/suspicious/noExplicitAny: <explanation>
  actualProperties: any,
  definitions: CloudFormationSchema['definitions'] | undefined,
  options: CreateResourceTableItemsOption,
): ResourceTableItem[] | undefined {
  const result: ResourceTableItem[] = [];
  for (const [definitionKey, definitionValue] of Object.entries(childrenProperties)) {
    // $refを含む場合は、参照プロパティを取得する
    if ('$ref' in definitionValue) {
      const childItem = parseReferencePropertyToTableItem(
        parentId,
        definitionKey,
        definitionValue,
        actualProperties,
        definitions,
        options,
      );
      if (childItem) {
        result.push(childItem);
      }
      continue;
    }

    // $refを含まない場合は、定義されたプロパティを取得する
    const childItem = parseDefinedPropertyToTableItem(
      parentId,
      definitionKey,
      definitionValue,
      actualProperties,
      definitions,
      options,
    );
    result.push(childItem);
  }
  return result.length === 0 ? undefined : result;
}

/**
 * 実際のプロパティを取得する
 * @param propertyKey プロパティのキー名
 * @param actualProperties 実際のテンプレートに定義されているプロパティ
 * @returns プロパティの実際の値
 */
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

/**
 * 疑似プロパティかどうかを判定する
 * @param property プロパティ名
 */
function isPseudoProperty(property: string): boolean {
  // biome-ignore lint/suspicious/noExplicitAny: <explanation>
  return pseudoProperties.includes(property as any);
}

/**
 * 組込み関数かどうかを判定する
 * @param actualProperty 実際のテンプレートに定義されているプロパティ
 * @returns 組込み関数の名前
 */
export function isIntrinsicFunction(
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

/**
 * 組込み関数の値を文字列化する
 * @param intrinsic 組込み関数
 * @param actualProperty 実際のテンプレートに定義されているプロパティ
 * @param options 組み込み関数用のデータ（ParameterとResourceList）
 * @returns 文字列化されたプロパティの値
 */
export function convertIntrinsicFunctionValue(
  intrinsic: IntrinsicFunction,
  // biome-ignore lint/suspicious/noExplicitAny: <explanation>
  actualProperty: any,
  options: CreateResourceTableItemsOption,
): NonNullable<Exclude<ResourceTableItem['value'], { type: 'array' }>> {
  switch (intrinsic) {
    case 'Fn::Base64': {
      return {
        type: 'value',
        value: JSON.stringify(actualProperty, undefined, '　'),
      };
    }
    case 'Fn::Cidr': {
      return {
        type: 'value',
        value: JSON.stringify(actualProperty, undefined, '　'),
      };
    }
    case 'Fn::And': {
      return {
        type: 'value',
        value: JSON.stringify(actualProperty, undefined, '　'),
      };
    }
    case 'Fn::Equals': {
      return {
        type: 'value',
        value: JSON.stringify(actualProperty, undefined, '　'),
      };
    }
    case 'Fn::If': {
      return {
        type: 'value',
        value: JSON.stringify(actualProperty, undefined, '　'),
      };
    }
    case 'Fn::Not': {
      return {
        type: 'value',
        value: JSON.stringify(actualProperty, undefined, '　'),
      };
    }
    case 'Fn::Or': {
      return {
        type: 'value',
        value: JSON.stringify(actualProperty, undefined, '　'),
      };
    }
    case 'Fn::FindInMap': {
      return {
        type: 'value',
        value: JSON.stringify(actualProperty, undefined, '　'),
      };
    }
    case 'Fn::ForEach': {
      return {
        type: 'value',
        value: JSON.stringify(actualProperty, undefined, '　'),
      };
    }
    case 'Fn::GetAtt': {
      const value = actualProperty['Fn::GetAtt'];

      if (value[0] in options.resources) {
        // <Fn::GetAtt: 論理ID.attr>
        const tabId = hrefBuilder({
          type: 'resource',
          stackId: options.stackId,
          sectionGroupName: options.stackName,
          serviceName: options.resources[value[0]].serviceName,
          resourceType: options.resources[value[0]].recourseType,
        });
        return {
          type: 'resource',
          tabId: tabId,
          stackId: options.stackId,
          stackName: options.stackName,
          serviceName: options.resources[value[0]].serviceName,
          resourceName: options.resources[value[0]].recourseType,
          selectedLogicalId: value[0],
          value: `<Fn::GetAtt: ${value[0]}.${value[1]}>`,
        };
      }
      return {
        type: 'value',
        value: JSON.stringify(actualProperty, undefined, '　'),
      };
    }
    case 'Fn::GetAZs': {
      return {
        type: 'value',
        value: JSON.stringify(actualProperty, undefined, '　'),
      };
    }
    case 'Fn::ImportValue': {
      const value = actualProperty['Fn::ImportValue'];

      if (value in options.externalResources) {
        // <Fn::ImportValue: ExportName>
        const tabId = hrefBuilder({
          type: 'overview',
          stackId: options.externalResources[value].stackId,
          sectionGroupName: options.externalResources[value].stackName,
        });
        return {
          type: 'overview',
          tabId: tabId,
          stackId: options.externalResources[value].stackId,
          stackName: options.externalResources[value].stackName,
          description: '',
          value: `<Fn::ImportValue: ${value}>`,
        };
      }

      return {
        type: 'value',
        value: JSON.stringify(actualProperty, undefined, '　'),
      };
    }
    case 'Fn::Join': {
      const delimiter = actualProperty['Fn::Join'][0];
      // biome-ignore lint/suspicious/noExplicitAny: <explanation>
      const pieces = actualProperty['Fn::Join'][1].map((value: any) => {
        // 文字列結合する要素の中にも組込み関数が含まれる場合があるので、再帰的に処理する
        const intrinsic = isIntrinsicFunction(value);
        if (intrinsic !== undefined) {
          return convertIntrinsicFunctionValue(intrinsic, value, options);
        }
        return value;
      });
      return {
        type: 'value',
        value: pieces.join(delimiter),
      };
    }
    case 'Fn::Length': {
      return {
        type: 'value',
        value: JSON.stringify(actualProperty, undefined, '　'),
      };
    }
    case 'Fn::Select': {
      return {
        type: 'value',
        value: JSON.stringify(actualProperty, undefined, '　'),
      };
    }
    case 'Fn::Split': {
      return {
        type: 'value',
        value: JSON.stringify(actualProperty, undefined, '　'),
      };
    }
    case 'Fn::Sub': {
      return {
        type: 'value',
        value: JSON.stringify(actualProperty, undefined, '　'),
      };
    }
    case 'Fn::ToJsonString': {
      return {
        type: 'value',
        value: JSON.stringify(actualProperty, undefined, '　'),
      };
    }
    case 'Fn::Transform': {
      return {
        type: 'value',
        value: JSON.stringify(actualProperty, undefined, '　'),
      };
    }
    case 'Ref': {
      let value = actualProperty.Ref;

      // Refの値が組込み関数の場合は、再帰的に処理する
      const intrinsic = isIntrinsicFunction(value);
      if (intrinsic !== undefined) {
        value = convertIntrinsicFunctionValue(intrinsic, value, options);
      }

      if (isPseudoProperty(value)) {
        // 疑似パラメータ：<疑似パラメータ>
        return {
          type: 'value',
          value: `<${value}>`,
        };
      }

      if (value in options.resources) {
        // 論理ID：<Ref: 論理ID>
        const tabId = hrefBuilder({
          type: 'resource',
          stackId: options.stackId,
          sectionGroupName: options.stackName,
          serviceName: options.resources[value].serviceName,
          resourceType: options.resources[value].recourseType,
        });
        return {
          type: 'resource',
          tabId: tabId,
          stackId: options.stackId,
          stackName: options.stackName,
          serviceName: options.resources[value].serviceName,
          resourceName: options.resources[value].recourseType,
          selectedLogicalId: value,
          value: `<Ref LogicalId: ${value}>`,
        };
      }

      if (options.parameters.includes(value)) {
        // パラメータ：<Ref: パラメータ>
        const tabId = hrefBuilder({
          type: 'overview',
          stackId: options.stackId,
          sectionGroupName: options.stackName,
        });
        return {
          type: 'overview',
          tabId: tabId,
          stackId: options.stackId,
          stackName: options.stackName,
          description: '',
          value: `<Ref: ${value}>`,
        };
      }

      // 疑似パラメータ以外：<Ref: 論理ID or Parameter>
      return {
        type: 'value',
        value: `<Ref: ${value}>`,
      };
    }
  }
}

/**
 * propertyに応じた値を文字列化する
 * @param property プロパティ定義
 * @param actualProperty 実際のテンプレートに定義されているプロパティ
 * @param options 組み込み関数用のデータ（ParameterとResourceList）
 * @returns 文字列化されたプロパティの値
 */
function convertValue(
  property: DefinedProperty,
  // biome-ignore lint/suspicious/noExplicitAny: <explanation>
  actualProperty: any,
  options: CreateResourceTableItemsOption,
): ResourceTableItem['value'] {
  if (actualProperty === undefined) {
    return undefined;
  }

  // プロパティが組込み関数の場合
  const intrinsic = isIntrinsicFunction(actualProperty);
  if (intrinsic !== undefined) {
    return convertIntrinsicFunctionValue(intrinsic, actualProperty, options);
  }

  // プロパティがプリミティブな型の場合はそのまま返す
  if (isJsonSchemaPrimitiveType(property.type)) {
    return {
      type: 'value',
      value: actualProperty,
    };
  }

  // プロパティが配列の場合
  if (Array.isArray(actualProperty)) {
    if (actualProperty.length !== 0) {
      const values = actualProperty.map((item) => {
        const itemType = typeof item;

        // 値がオブジェクトの場合
        if (itemType === 'object') {
          const intrinsic = isIntrinsicFunction(item);
          if (intrinsic !== undefined) {
            return convertIntrinsicFunctionValue(intrinsic, item, options);
          }
        }
        return {
          type: 'value' as const,
          value: JSON.stringify(item, undefined, '　'),
        };
      });
      return {
        type: 'array',
        value: values,
      };
    }
  }

  // これから以外の場合はJSON.stringifyで文字列化する
  return {
    type: 'value',
    value: JSON.stringify(actualProperty, undefined, '　'),
  };
}

/**
 * `property.type`を型の文字列に変換する
 * @param property `$ref`を含まないプロパティ
 * @returns 型の文字列
 */
function convertType(property: DefinedProperty): string {
  // タイプが配列の場合は、配列要素を結合する
  if (Array.isArray(property.type)) {
    return property.type.join('\n');
  }

  // タイプが文字列結でかつenumが定義されている場合は、enumの値を結合する
  if (property.type === 'string') {
    if (property.enum) {
      return property.enum.map((item) => item.toString()).join('\n');
    }
  }
  if (property.type === 'array') {
    // プロパティが配列で、itemsが定義されている場合
    if (property.items && !('$ref' in property.items)) {
      // itemsが配列の場合は、子要素のタイプを再帰的に変換し結合する
      if (isJsonSchemaPrimitiveType(property.items.type)) {
        const type = convertType(property.items);
        if (type.includes('\n')) {
          return `( \n　${type.split('\n').join('\n　')}\n) []`;
        }
        return `${type}[]`;
      }
    }
  }
  return property.type;
}
