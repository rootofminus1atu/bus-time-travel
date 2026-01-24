import pulumi_aws as aws
import json


def lambda_logs_policy(name: str):
    return aws.iam.Policy(
        name,
        policy=json.dumps({
            "Version": "2012-10-17",
            "Statement": [{
                "Effect": "Allow",
                "Action": [
                    "logs:CreateLogGroup",
                    "logs:CreateLogStream",
                    "logs:PutLogEvents",
                ],
                "Resource": "arn:aws:logs:*:*:*",
            }]
        })
    )

def s3_write_policy(name: str, bucket_arn):
    return aws.iam.Policy(
        name,
        policy=bucket_arn.apply(lambda arn: json.dumps({
            "Version": "2012-10-17",
            "Statement": [{
                "Effect": "Allow",
                "Action": "s3:PutObject",
                "Resource": f"{arn}/*",
            }]
        }))
    )

def s3_read_policy(name: str, bucket_arn):
    return aws.iam.Policy(
        name,
        policy=bucket_arn.apply(lambda arn: json.dumps({
            "Version": "2012-10-17",
            "Statement": [{
                "Effect": "Allow",
                "Action": "s3:GetObject",
                "Resource": f"{arn}/*",
            }]
        }))
    )


def lambda_role(name: str):
    return aws.iam.Role(
        name,
        assume_role_policy=json.dumps({
            "Version": "2012-10-17",
            "Statement": [{
                "Effect": "Allow",
                "Principal": {"Service": "lambda.amazonaws.com"},
                "Action": "sts:AssumeRole",
            }]
        })
    )

def attach(role, policy, name):
    aws.iam.RolePolicyAttachment(
        name,
        role=role.id,
        policy_arn=policy.arn,
    )


logs_policy = lambda_logs_policy("shared-lambda-logs")


def write_to_bucket_role(name: str, bucket):
    role = lambda_role(f"{name}-role")

    policy = s3_write_policy(f"{name}-s3-write", bucket.arn)

    attach(role, logs_policy, f"{name}-logs")
    attach(role, policy, f"{name}-write-attach")

    return role

def read_from_bucket_role(name: str, bucket):
    role = lambda_role(f"{name}-role")

    policy = s3_read_policy(f"{name}-s3-read", bucket.arn)

    attach(role, logs_policy, f"{name}-logs")
    attach(role, policy, f"{name}-read-attach")

    return role
    