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
    const itemType = covertType(value);
    const item: ResourceTableItem = {
      id: `/properties/${key}`,
      property: key,
      type: itemType,
      description: value.description || '',
      value: '',
    };

    if (value.items) {
      item.children = [];
      if ('$ref' in value.items && schema.definitions) {
        const definition = value.items.$ref.replace('#/definitions/', '');
        const definitionSchema = schema.definitions[definition];
        for (const [definitionKey, definitionValue] of Object.entries(
          definitionSchema.properties,
        )) {
          const childType = covertType(definitionValue);
          const child: ResourceTableItem = {
            id: `/properties/${key}/${definitionKey}`,
            property: definitionKey,
            type: childType,
            description: definitionValue.description || '',
            value: '',
          };
          item.children.push(child);
        }
      } else {
      }
    }

    items.push(item);
  }
  return items;
};

function covertType(property: Property): string {
  console.log('property', property);
  if (typeof property.type === 'string') {
    if (property.enum) {
      return property.enum.map((item) => item.toString()).join(' | ');
    }
    return property.type;
  }
  console.log('property.type', typeof property.type, property.type);
  return property.type.join(' | ');
}
