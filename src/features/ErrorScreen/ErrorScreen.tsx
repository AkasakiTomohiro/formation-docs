import { ContentLayout, TextContent } from '@cloudscape-design/components';

export const ErrorScreen = () => {
  return (
    <ContentLayout defaultPadding>
      <TextContent>
        起動時に必要なリソースのダウンロードに失敗しました。
        <br />
        オンラインであることをご確認のうえ、再度アプリを起動しなおしてください。
      </TextContent>
    </ContentLayout>
  );
};
