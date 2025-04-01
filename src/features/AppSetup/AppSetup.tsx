import { useAppConfig } from '../../hooks/useAppConfig';

export const AppSetup = (): JSX.Element => {
  const { appConfig } = useAppConfig();
  if (appConfig === null) {
    return <></>;
  }
  if (appConfig.initialized === false) {
    return <>初回セットアップ</>;
  }
  return (
    <div>
      <h1>App Setup</h1>
      <p>Configure your application settings here.</p>
    </div>
  );
};
