import { Construct } from 'constructs';
import { TerraformOutput } from 'cdktf';
//import { AwsProvider, LambdaFunction, CloudwatchLogGroup } from '@cdktf/provider-aws';
//import { AwsProvider } from '@cdktf/provider-aws/lib/provider';
import {AwsProvider } from '../../.gen/providers/aws/provider'
//import { AwsProvider } from '../../../.gen/providers/aws/provider'
import { CloudwatchLogGroup } from '@cdktf/provider-aws/lib/cloudwatch-log-group';
import { LambdaFunction } from '@cdktf/provider-aws/lib/lambda-function';
//import { Lambda, LambdaConfig } from '../../../.gen/modules/lambda'
import { role as CreateRole } from './role';
import { LambdaCommonVariables } from './variables';
import { TrulyTags } from '../utils/tags';
import { calculateHashFromFile } from '../utils/hasher';
import { ENV_PROD } from '../utils/constants';

export interface LambdaLoginProps {
  // Define your module's properties here, including required parameters and defaults. 
  // For instance, I've added regions and the Lambda function name.
  regions: string[],
  functionName: string,
  environment: string,
  variables: LambdaCommonVariables,
  tags: TrulyTags,
  provs: Map<string,AwsProvider>
  // ...additional properties as needed...
}

export class LambdaLogin extends Construct {
  constructor(scope: Construct, name: string, props: LambdaLoginProps) {
    super(scope, name);
    const role = CreateRole(scope).arn;

    let login_tags = { ...props.tags, service: 'login' }

    const labelLogin = 'lambdaLogin';

    props.regions.forEach(async region => {
      const provider = props.provs.get(region); 
      /*new AwsProvider(this, `provider_${region}`, { 
        region, 
        profile:'personal', 
        alias: region.replace(/-/g, '_')  
      });*/

      //const logGroup = 
      new CloudwatchLogGroup(this, `logGroup_${labelLogin}_${region}`, {
        name: `/aws/lambda/${props.functionName}-${region}`,
        retentionInDays: 1,
        tags: login_tags,
        provider
      });

      const lambdaPathFileName = props.variables.path + props.variables.file;
      const lambdaLogin = new LambdaFunction(this, `${labelLogin}_${region}`, {
        functionName: `${props.functionName}-${region}`,
        role,
        architectures: [props.variables.architecture],
        handler: props.variables.function_handler,
        runtime: props.variables.runtime,
        filename: lambdaPathFileName,
        memorySize: 512,
        timeout: 60,
        sourceCodeHash: await calculateHashFromFile(lambdaPathFileName),
        tracingConfig: { mode: 'Active' },
        environment: {
          variables: {
            'ENVIRONMENT': props.environment,
            'RUST_LOG': props.environment === ENV_PROD ? 'cargo_lambda=error' : 'cargo_lambda=info',
            'JWT_TOKEN_TIME_EXP_HOURS': props.environment === ENV_PROD ? '1' : '8',
            'RUST_BACKTRACE': props.environment === ENV_PROD ? '0' : 'full',
          }
        },
        tags: login_tags,
        provider
      });

      new TerraformOutput(this, `${labelLogin}-${region}`, {
        value: lambdaLogin.arn,
      });

    });
  }
}
