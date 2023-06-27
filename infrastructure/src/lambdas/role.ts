import { IamRole } from '@cdktf/provider-aws/lib/iam-role';
import { Construct } from 'constructs';


export const role = (ctx: Construct) => {
    const lambdaExecutionRole = new IamRole(ctx, 'LambdaExecutionRole', {
        name: 'lambdaExecutionRole',
        assumeRolePolicy: JSON.stringify({
            Version: "2012-10-17",
            Statement: [
                {
                    Action: "sts:AssumeRole",
                    Principal: {
                        Service: "lambda.amazonaws.com"
                    },
                    Effect: "Allow",
                    Sid: ""
                }
            ]
        }),
    });
    return lambdaExecutionRole;
};

