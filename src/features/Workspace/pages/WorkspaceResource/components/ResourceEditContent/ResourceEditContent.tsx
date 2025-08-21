import { Container, FormField, SpaceBetween, Textarea } from '@cloudscape-design/components';
import type { ContainerProps } from '@cloudscape-design/components';
import { z } from 'zod';

export const ResourceEditValidator = z.object({
  description: z.string().max(256),
});
export type ResourceEditType = z.infer<typeof ResourceEditValidator>;

export type ResourceEditContentProps = {
  /**
   * リソースのdescription
   */
  description: string;

  /**
   * descriptionを更新する関数
   */
  setDescription: React.Dispatch<React.SetStateAction<string>>;

  /**
   * コンテナのヘッダー
   */
  header?: ContainerProps['header'];
};

export const ResourceEditContent = ({ description, setDescription, header }: ResourceEditContentProps): JSX.Element => {
  return (
    <Container header={header}>
      <SpaceBetween direction="vertical" size="m">
        <FormField label="description">
          {/* // TODO: 256文字以上は入力できないように修正 */}
          <Textarea onChange={(event) => setDescription(event.detail.value)} value={description} />
        </FormField>
      </SpaceBetween>
    </Container>
  );
};
