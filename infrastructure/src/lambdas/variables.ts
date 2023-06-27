
export interface LambdaCommonVariables {
    function_handler: string,
    runtime: string,
    architecture: string,
    file: string,
    path: string,
}

export const NewDefaultVariables = (path:string): LambdaCommonVariables => {
    return {
        function_handler: 'function_handler',
        runtime: 'provided.al2',
        architecture: 'arm64',
        file: 'bootstrap.zip',
        path
    }
}
