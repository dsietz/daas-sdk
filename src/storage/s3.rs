use crate::errors::daaserror::DaaSStorageError;
use rusoto_core::Region;
use rusoto_s3::{S3, S3Client, PutObjectRequest, StreamingBody};

/// Credentials are read from the environment vcariables AWS_ACCESS_KEY_ID and AWS_SECRET_ACCESS_KEY

/// Represents a facilitator for managing a S3 Bucket and it's content
#[derive(Debug, Clone)]
pub struct S3BucketMngr {
    /// The enum that represents the AWS region of the bucket, (e.g.: Region::UsEast1) - See rusoto_core documentation for further information
    pub region: Region,
    /// The name of the S3 Bucket
    pub bucket: String,
    /// The AWS ARN of the S3 Bucket
    pub arn: String,
}

impl S3BucketMngr {
    /// Constructs a S3BucketMngr object
    /// 
    /// # Arguments
    /// 
    /// * region: Region - The enum that represents the AWS region of the bucket, (e.g.: Region::UsEast1) - See rusoto_core documentation for further information.</br>
    /// * bucket_name: String - The name of the S3 bucket.</br>
    /// 
    /// #Example
    ///
    /// ```
    /// extern crate daas;
    ///
    /// use rusoto_core::Region;
    /// use daas::storage::s3::S3BucketMngr;
    ///
    /// fn main() {
    ///    let mut bckt = S3BucketMngr::new(Region::UsEast1, "iapp-daas-test-bucket".to_string());
    ///
    ///    assert_eq!(bckt.bucket, "iapp-daas-test-bucket".to_string());
    /// }
    /// ```
    pub fn new(region: Region, bucket_name: String) -> S3BucketMngr {
        S3BucketMngr {
            region: region,
            bucket: bucket_name.clone(),
            arn: format!("arn:aws:s3:::{}",bucket_name).to_string(),
        }
    }

    /// Constructs a S3BucketMngr object based on it's ARN
    /// 
    /// # Arguments
    /// 
    /// * region: Region - The enum that represents the AWS region of the bucket, (e.g.: Region::UsEast1) - See rusoto_core documentation for further information.</br>
    /// * bucket_arn: String - The arn of the S3 bucket.</br>
    /// 
    /// #Example
    ///
    /// ```
    /// extern crate daas;
    ///
    /// use rusoto_core::Region;
    /// use daas::storage::s3::S3BucketMngr;
    ///
    /// fn main() {
    ///    let mut bckt = S3BucketMngr::from_arn(Region::UsEast1, "arn:aws:s3:::iapp-daas-test-bucket".to_string());
    ///
    ///    assert_eq!(bckt.bucket, "iapp-daas-test-bucket".to_string());
    /// }
    /// ```
    pub fn from_arn(region: Region, bucket_arn: String) -> S3BucketMngr {
        let mut arn = S3BucketMngr::parse_arn(bucket_arn.clone());
		S3BucketMngr {
            region: region,
            bucket: arn[5].take().unwrap(),
            arn: bucket_arn,
        }
    }

    /// Parses an ARN for a S3 bucket into its components
    /// (see https://docs.aws.amazon.com/general/latest/gr/aws-arns-and-namespaces.html)
    /// 
    /// # Arguments
    /// 
    /// * arn: String - The arn of the S3 bucket.</br>
    /// 
    /// #Example
    ///
    /// ```
    /// extern crate daas;
    ///
    /// use daas::storage::s3::S3BucketMngr;
    ///
    /// fn main() {
    ///    let mut arn_parts = S3BucketMngr::parse_arn("arn:aws:s3:::iapp-daas-test-bucket".to_string());
    ///
    ///    assert_eq!(arn_parts[5].take().unwrap(), "iapp-daas-test-bucket".to_string());
    /// }
    /// ```
    pub fn parse_arn(arn: String) -> Vec<Option<String>>{
        let mut parts = Vec::new();
        
        for part in arn.split(":").collect::<Vec<&str>>().iter() {
            if part.len() == 0 {
                parts.push(None);
            }
            else {
                parts.push(Some(part.to_string()));
            }
        }
    
        parts
    }

    /// Uploads a file to the S3 Bucket
    ///
    /// # Arguments
    /// 
    /// * content_key: String - The S3 Bucket prefix key to use for the document, (e.g.: "myfolder/myfile.txt").</br>
    /// * content: StreamingBody - The ByteStream that is the content of the file.</br>
    /// 
    /// #Example
    ///
    /// ```
    /// extern crate daas;
    /// extern crate rusoto_s3;
    ///
    /// use daas::storage::s3::S3BucketMngr;
    /// use rusoto_core::Region;
    /// use rusoto_s3::{StreamingBody};
    ///
    /// fn main() {
    ///     let bckt = S3BucketMngr::new(Region::UsEast1, "iapp-daas-test-bucket".to_string());
    ///     let content: StreamingBody = String::from("this is a message....").into_bytes().into();
    ///
    ///     match bckt.upload_file("tmp/mystuff/new-record2.txt".to_string(), content) {
    ///         Ok(_y) => assert!(true),
    ///         Err(err) => panic!("{:?}", err),
    ///     }
    /// }
    /// ```
    pub fn upload_file(self, content_key: String, content: StreamingBody) -> Result<i8, DaaSStorageError>{
        let s3_client = S3Client::new(Region::UsEast1);
        let req = PutObjectRequest {
            bucket: self.bucket,
            key: content_key,
            body: Some(content),
            acl: Some("private".to_string()),
            ..Default::default()
        };
    
        match s3_client.put_object(req).sync() {
            Ok(_t) => Ok(1),
            Err(_err) => Err(DaaSStorageError::UpsertError),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_arn(){
        let bckt = S3BucketMngr::from_arn(Region::UsEast1, "arn:aws:s3:::iapp-daas-test-bucket".to_string());
        
        assert_eq!(bckt.bucket, "iapp-daas-test-bucket".to_string());
        assert_eq!(bckt.arn, "arn:aws:s3:::iapp-daas-test-bucket".to_string());
        assert_eq!(bckt.region, Region::UsEast1);
    }

    #[test]
    fn test_new_s3bucketmngr() {
        let bckt = S3BucketMngr::new(Region::UsEast1, "iapp-daas-test-bucket".to_string());

        assert_eq!(bckt.bucket, "iapp-daas-test-bucket".to_string());
        assert_eq!(bckt.arn, "arn:aws:s3:::iapp-daas-test-bucket".to_string());
        assert_eq!(bckt.region, Region::UsEast1);
    }

    #[test]
    fn test_upload_file() {
        let bckt = S3BucketMngr::new(Region::UsEast1, "iapp-daas-test-bucket".to_string());
        let content: StreamingBody = String::from("this is a message....").into_bytes().into();

        let rslt = bckt.upload_file("tmp/mystuff/new-record2.txt".to_string(), content).unwrap();
        assert_eq!(rslt, 1);
    }
}