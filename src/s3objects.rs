use aws_config::meta::region::RegionProviderChain;
use aws_sdk_s3 as s3;
use aws_smithy_types;

pub struct S3Result {
    pub path: String,
    pub label: String,
    pub size: i64,
    pub is_directory: bool,
    pub is_matched: bool,
    pub last_modified: String,
}

impl S3Result {
    fn new(path: String, size: i64, last_modified: Option<s3::types::DateTime>) -> S3Result {
        let path_parts = path.split("/").collect::<Vec<&str>>();
        let is_directory = match path.chars().last().unwrap() {
            '/' => true,
            _ => false,
        };

        let label = match is_directory {
            true => path_parts[path_parts.len() - 2..path_parts.len()]
                .join("/")
                .to_string(),
            false => path_parts.last().unwrap().to_string(),
        };

        let format = aws_smithy_types::date_time::Format::DateTime;

        let date_str = match last_modified {
            None => "_".to_string(),
            Some(date) => date.fmt(format).unwrap(),
        };

        S3Result {
            path: path.clone(),
            size: size,
            label: label,
            is_directory: is_directory,
            is_matched: true,
            last_modified: date_str,
        }
    }
}

async fn get_results(
    client: &s3::Client,
    bucket_name: String,
    prefix: String,
) -> Result<Vec<S3Result>, s3::Error> {
    let objects = client
        .list_objects_v2()
        .bucket(bucket_name)
        .prefix(prefix)
        //.max_keys(200)
        .delimiter("/")
        .send()
        .await?;

    let mut result: Vec<S3Result> = Vec::new();

    for obj in objects.common_prefixes().unwrap_or_default() {
        result.push(S3Result::new(String::from(obj.prefix().unwrap()), 0, None));
    }
    for obj in objects.contents().unwrap_or_default() {
        result.push(S3Result::new(
            String::from(obj.key().unwrap()),
            obj.size(),
            Some(*obj.last_modified().unwrap()),
        ));
    }
    return Ok(result);
}

pub async fn get_objects(bucket_name: &str, path: &str) -> Result<Vec<S3Result>, s3::Error> {
    let region_provider = RegionProviderChain::default_provider().or_else("us-east-1");
    let config = aws_config::from_env().region(region_provider).load().await;
    let client = s3::Client::new(&config);

    let bucket_name = String::from(bucket_name);
    let prefix = String::from(path);
    let results = get_results(&client, bucket_name, prefix).await?;

    Ok(results)
}
