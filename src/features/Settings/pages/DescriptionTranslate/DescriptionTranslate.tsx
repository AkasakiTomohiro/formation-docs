import { useEffect, useState } from 'react';
import { useNavigate, useParams } from 'react-router';
import { getCloudFormationSchema } from '../../../../invoke/CloudFormationSchema';
import { createExpandedItems } from '../../../Workspace/pages/WorkspaceResource/components/PropertyTable';
import { createResourceTableItems } from '../../../Workspace/pages/WorkspaceResource/lib/CreateResourceTableItems';
import {
  DescriptionTranslatePresentation,
  type DescriptionTranslatePresentationProps,
  type TableItem,
} from './DescriptionTranslate.presentation';
import type { CloudFormationSchema } from '../../../Workspace/pages/WorkspaceResource/components/types/CloudFormationSchema';

export const DescriptionTranslate = () => {
  const navigate = useNavigate();
  const { serviceName, resourceType } = useParams() as { serviceName: string; resourceType: string };
  const [properties, setProperties] = useState<DescriptionTranslatePresentationProps['properties']>([]);
  const [expandedItems, setExpandedItems] = useState<any>(); // ネストされたプロパティの展開状態を管理するためのステート
  const [editingTranslations, setEditingTranslations] = useState<Record<string, string>>({});

  useEffect(() => {
    getCloudFormationSchema(serviceName, resourceType).then((schema) => {
      const schemaParsed = JSON.parse(schema) as CloudFormationSchema;
      const resourceTableItems = createResourceTableItems(schemaParsed, {});
      setProperties(resourceTableItems);
      const expandedItems = createExpandedItems(resourceTableItems);
      setExpandedItems(expandedItems);
    });
  }, [serviceName, resourceType]);

  const onChangeTranslation: DescriptionTranslatePresentationProps['onChangeTranslation'] =
    (item) =>
    ({ detail }) => {
      setEditingTranslations((prev) => ({ ...prev, [item.id]: detail.value }));
      console.log(editingTranslations);
    };

  const onClickCancel = () => {
    setEditingTranslations({});
    navigate('/settings');
  };

  return (
    <DescriptionTranslatePresentation
      properties={properties}
      expandableRows={{
        getItemChildren: (item) => item.children ?? [],
        isItemExpandable: (item) => Boolean(item.children),
        expandedItems: expandedItems,
        onExpandableItemToggle: ({ detail }) =>
          setExpandedItems((prev: TableItem[] | undefined) => {
            const next = new Set((prev ?? []).map((item) => item.id));
            detail.expanded ? next.add(detail.item.id) : next.delete(detail.item.id);
            return [...next].map((id) => ({ id }));
          }),
      }}
      onClickSave={() => {}}
      onClickCancel={onClickCancel}
      editingTranslations={editingTranslations}
      onChangeTranslation={onChangeTranslation}
      serviceName={serviceName}
      resourceType={resourceType}
    />
  );
};
