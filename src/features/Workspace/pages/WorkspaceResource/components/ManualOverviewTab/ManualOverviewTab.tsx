import { useState } from 'react';

import { Box, Button, ContentLayout, Header, SpaceBetween, TextContent } from '@cloudscape-design/components';

import { ManualResourceCreateContent } from './components/ManualResourceCreateContent';

export function buildManualOverviewTabName(): string {
  return '手動管理リソース';
}

export const ManualOverviewTab = (): JSX.Element => {
  const [registering, setRegistering] = useState<boolean>(false);

  return (
    <ContentLayout header={<Header variant="h1">手動管理リソース</Header>}>
      {/* Overview */}
      {registering === false && (
        <SpaceBetween direction="vertical" size="s">
          <TextContent>
            <p>CloudFormationで管理していないAWSリソースの設定を管理する機能です。</p>
          </TextContent>
          <Box float="right">
            <Button variant="primary" onClick={() => setRegistering(true)}>
              新規リソース作成
            </Button>
          </Box>
        </SpaceBetween>
      )}
      {/* リソース作成コンポーネント */}
      {registering === true && <ManualResourceCreateContent setRegistering={setRegistering} />}
    </ContentLayout>
  );
};
