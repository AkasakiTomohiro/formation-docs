#!/usr/bin/env node
import * as cdk from 'aws-cdk-lib';
import { SampleAppStack } from '../lib/sample-app-stack';
import { SampleFuncStack } from '../lib/sample-func-stack';

const app = new cdk.App();
new SampleAppStack(app, 'SampleAppStack', {
  synthesizer: new cdk.DefaultStackSynthesizer({
    generateBootstrapVersionRule: false,
  }),
});
new SampleFuncStack(app, 'SampleFuncStack', {
  synthesizer: new cdk.DefaultStackSynthesizer({
    generateBootstrapVersionRule: false,
  }),
});
