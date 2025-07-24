import { CfnParameter, Duration, Stack } from 'aws-cdk-lib';
import * as sns from 'aws-cdk-lib/aws-sns';
import * as subs from 'aws-cdk-lib/aws-sns-subscriptions';
import * as sqs from 'aws-cdk-lib/aws-sqs';

import type { StackProps } from 'aws-cdk-lib';
import type { Construct } from 'constructs';

export class SampleAppStack extends Stack {
  public readonly queue: sqs.Queue;

  constructor(scope: Construct, id: string, props?: StackProps) {
    super(scope, id, props);

    const queue = new sqs.Queue(this, 'SampleAppQueue', {
      visibilityTimeout: Duration.seconds(300),
      fifo: true,
    });
    this.queue = queue;

    new CfnParameter(this, 'SampleParameter', {
      type: 'String',
      description: 'A sample parameter for the stack',
    });

    const topic = new sns.Topic(this, 'SampleAppTopic');

    topic.addSubscription(new subs.SqsSubscription(queue));
  }
}
