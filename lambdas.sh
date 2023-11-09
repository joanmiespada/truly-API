#!/bin/zsh

lambdas='[
        {
            "name": "license_lambda",
            "version": "0.0.4",
            "path": "lambda_license/image/Dockerfile",
            "description": "License lambda: manage assets"
        },{
            "name": "admin_lambda",
            "version": "0.0.4",
            "path": "lambda_admin/image/Dockerfile",
            "description": "Admin lambda: manage operation with high privilegies"
        },{
            "name": "login_lambda",
            "version": "0.0.4",
            "path": "lambda_login/image/Dockerfile",
            "description": "Login lambda: manage login and signups"
        },{
            "name": "user_lambda",
            "version": "0.0.4",
            "path": "lambda_user/image/Dockerfile",
            "description": "User lambda: manage user crud ops"
        },{
            "name": "after_hash_lambda",
            "version": "0.0.4",
            "path": "lambda_after_hash/image/Dockerfile",
            "description": "After hash lambda: manage asset state after hashes computation"
        },{
            "name": "error_lambda",
            "version": "0.0.4",
            "path": "lambda_error/image/Dockerfile",
            "description": "Error lambda: capture errors "
        },{
            "name": "alert_similars_lambda",
            "version": "0.0.4",
            "path": "lambda_alert_similars/image/Dockerfile",
            "description": "Alert Similar lambda: get alerts from matchapi about new matches among assets"
        },{
            "name": "notifications_lambda",
            "version": "0.0.4",
            "path": "lambda_notifications/image/Dockerfile",
            "description": "Notifications lambda: send notifications "
        }
    ]'

parse_lambda_names() {
    for lambda in "${lambdas_to_build[@]}"; do
        build_specific_lambda[$lambda]=1
    done
}

