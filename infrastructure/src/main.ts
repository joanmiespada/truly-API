import { Construct } from 'constructs'
import { App, TerraformStack } from 'cdktf'
import { LambdaLogin, LambdaLoginProps } from './lambdas/lambda_login'
import { TrulyTags } from './utils/tags';
import { NewDefaultVariables } from './lambdas/variables';
import { ENV_PROD, ENV_STAGE } from './utils/constants';
//import { AwsProvider } from '@cdktf/provider-aws/lib/provider';
import {AwsProvider } from '../.gen/providers/aws/provider'

interface TrulyStackConfig {
  environment: string;
  regions: string[];
}

class TrulyStack extends TerraformStack {
  constructor(scope: Construct, id: string, config: TrulyStackConfig) {
    super(scope, id)

    let tags: TrulyTags = {
      project: 'truly',
      component: 'api',
    }
    const architecture = process.env.ARCHITECTURE || 'aarch64-linux-gnu'
    const baseLambdaPath = `../target/lambda_${architecture}`

    const provs: Map<string, AwsProvider> = new Map();
    config.regions.forEach(async region => {
      const aux = new AwsProvider(this, `provider_${region}`, {
        region,
        //profile: 'personal',
        alias: region.replace(/-/g, '_') ,
        accessKey: 'AKIASQMHSDFD5WBVZ6IS', 
        secretKey: 'zamKiqSSjLGkuEb4ZzgEZF+fErcbtoE+hkgGnkuS'
      });
      provs.set(region, aux);
    });


    const lambdaLoginFunctionName = 'lambda_login';
    const aux: LambdaLoginProps = {
      regions: config.regions,
      functionName: lambdaLoginFunctionName,
      environment: config.environment,
      variables: NewDefaultVariables(`${baseLambdaPath}/${lambdaLoginFunctionName}/`),
      tags,
      provs
    }
    //const lambda_login = 
    new LambdaLogin(this, lambdaLoginFunctionName, aux)

  }
}


const app = new App()
//if (process.env.NODE_ENV !== ENV_PROD) {
new TrulyStack(app, 'truly-api-stage', { environment: ENV_STAGE, regions: ['eu-west-1'] });
//} else {
new TrulyStack(app, 'truly-api-prod', { environment: ENV_PROD, regions: ['us-west-1', 'eu-central-1'] });
//}
app.synth()
