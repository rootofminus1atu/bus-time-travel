import pulumi
from roles import read_from_bucket_role
import pulumi_aws as aws
import os

bucket = aws.s3.Bucket("bus-time-travel")

reader_role = read_from_bucket_role("get_history_role", bucket)

get_history_lambda_path = os.path.abspath("../backend/target/lambda/bus_history")
get_history_lambda = aws.lambda_.Function(
    "get_history",
    runtime="provided.al2023",
    handler="bootstrap",
    role=reader_role.arn,
    code=pulumi.AssetArchive({ ".": pulumi.FileArchive(get_history_lambda_path) })
)