import {
  intrinsicFunctions,
  isJsonSchemaPrimitiveType,
  pseudoProperties,
} from '../components/types/CloudFormationSchema';

import type {
  CloudFormationSchema,
  DefinedProperty,
  IntrinsicFunction,
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

  // 子要素がある場合はvalueを空にし、ない場合はchildrenを削除する
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
  // ReferenceProperty場合はdefinitionsがundefinedになりえないため。処理を中断する
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

  // プロパティがオブジェクトの場合は、子要素を取得する
  if (definition.type === 'object') {
    result.children = [];

    // definitionに定義されるプロパティを取得する
    for (const [definitionKey, definitionValue] of Object.entries(
      definition.properties,
    )) {
      // $refを含む場合は再帰的に処理する
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

      // $refを含まない場合は、定義されたプロパティを取得する
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

  // 子要素がある場合はvalueを空にし、ない場合はchildrenを削除する
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

/**
 * 組込み関数の値を文字列化する
 * @param intrinsic 組込み関数
 * @param actualProperty 実際のテンプレートに定義されているプロパティ
 * @returns 文字列化されたプロパティの値
 */
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
        // 文字列結合する要素の中にも組込み関数が含まれる場合があるので、再帰的に処理する
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

      // Refの値が組込み関数の場合は、再帰的に処理する
      const intrinsic = isIntrinsicFunction(value);
      if (intrinsic !== undefined) {
        value = convertIntrinsicFunctionValue(intrinsic, value);
      }

      if (isPseudoProperty(value)) {
        // 疑似パラメータ：<疑似パラメータ>
        return `<${value}>`;
      }
      // 疑似パラメータ以外：<Ref: 論理ID or Parameter>
      return `<Ref: ${value}>`;
    }
  }
}

/**
 * propertyに応じた値を文字列化する
 * @param property プロパティ定義
 * @param actualProperty 実際のテンプレートに定義されているプロパティ
 * @returns 文字列化されたプロパティの値
 */
function covertValue(
  property: DefinedProperty,
  // biome-ignore lint/suspicious/noExplicitAny: <explanation>
  actualProperty: any,
): string {
  if (actualProperty === undefined) {
    return '';
  }

  // プロパティが組込み関数の場合
  const intrinsic = isIntrinsicFunction(actualProperty);
  if (intrinsic !== undefined) {
    return convertIntrinsicFunctionValue(intrinsic, actualProperty);
  }

  // プロパティがプリミティブな型の場合はそのまま返す
  if (isJsonSchemaPrimitiveType(property.type)) {
    return actualProperty;
  }

  // プロパティが配列かつ、値がプリミティブな場合
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
  // タイプが配列の場合は、配列要素を結合する
  if (Array.isArray(property.type)) {
    return property.type.join(' | ');
  }

  // タイプが文字列結でかつenumが定義されている場合は、enumの値を結合する
  if (property.type === 'string') {
    if (property.enum) {
      return property.enum.map((item) => item.toString()).join(' | ');
    }
  }
  if (property.type === 'array') {
    // プロパティが配列で、itemsが定義されている場合
    if (property.items && !('$ref' in property.items)) {
      // itemsが配列の場合は、子要素のタイプを再帰的に変換し結合する
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
