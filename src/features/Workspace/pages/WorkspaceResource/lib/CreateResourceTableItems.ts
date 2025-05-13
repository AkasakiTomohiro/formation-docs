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

export const createResourceTableItems = (
  schema: CloudFormationSchema,
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
        schema.definitions,
      );
      items.push(definedItem);
    }
  }
  return items;
};

function parseDefinedPropertyToTableItem(
  parentId: string,
  propertyKey: string,
  property: DefinedProperty,
  definitions?: CloudFormationSchema['definitions'],
): ResourceTableItem {
  const result: ResourceTableItem = {
    id: `${parentId}/${propertyKey}`,
    property: propertyKey,
    type: covertType(property),
    description: property.description || '',
    value: '',
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
          definitions,
        );
        result.children.push(childItem);
      }
    }
  }
  return result;
}

function parseReferencePropertyToTableItem(
  parentId: string,
  propertyKey: string,
  property: ReferenceProperty | ReferencePropertyWithDescription,
  definitions?: CloudFormationSchema['definitions'],
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
    value: '',
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
        definitions,
      );
      result.children.push(childItem);
    }
  }

  return result;
}

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
