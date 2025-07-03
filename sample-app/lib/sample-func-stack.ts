import path from 'node:path';
import { Duration, Stack } from 'aws-cdk-lib';
import { Vpc } from 'aws-cdk-lib/aws-ec2';
import * as lambda from 'aws-cdk-lib/aws-lambda';
import { Asset } from 'aws-cdk-lib/aws-s3-assets';

import type { StackProps } from 'aws-cdk-lib';
import type { Construct } from 'constructs';
import type { SampleAppStack } from './sample-app-stack';

const __dirname = import.meta.dirname;

interface SampleFuncStackProps extends StackProps {
  sampleAppStack: SampleAppStack;
}

export class SampleFuncStack extends Stack {
  constructor(scope: Construct, id: string, props: SampleFuncStackProps) {
    super(scope, id, props);

    const asset = new Asset(this, 'SampleFuncCode', {
      path: path.join(__dirname, '../func/sample-func.js'),
    });

    const vpc = new Vpc(this, 'SampleVpc');

    new lambda.Function(this, 'SampleFunc', {
      runtime: lambda.Runtime.NODEJS_22_X,
      handler: 'index.handler',
      code: lambda.Code.fromBucket(asset.bucket, asset.s3ObjectKey),
      description: 'Sample Lambda Function',
      filesystem: {
        config: {
          arn: '',
          localMountPath: '/mnt/efs',
        },
      },
      vpc: vpc,
      vpcSubnets: {
        subnets: vpc.privateSubnets,
      },
      environment: {
        QUEUE_ARN: props.sampleAppStack.queue.queueArn,
      },
      architecture: lambda.Architecture.ARM_64,
      timeout: Duration.seconds(10),
      loggingFormat: lambda.LoggingFormat.JSON,
      systemLogLevelV2: lambda.SystemLogLevel.INFO,
    });
  }
}
