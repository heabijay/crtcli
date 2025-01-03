use crate::app::{CrtClient, CrtClientGenericError, CrtDbType};

pub struct SqlScripts<'c>(&'c CrtClient);

impl<'c> SqlScripts<'c> {
    pub fn new(client: &'c CrtClient) -> Self {
        Self(client)
    }

    pub fn mark_package_as_not_changed(
        &self,
        package_uid: &str,
    ) -> Result<u64, CrtClientGenericError> {
        let query = match self.0.db_type()? {
            CrtDbType::MsSql => format!(
                r#"UPDATE "SysSchema" 
                SET "IsChanged" = 0, "IsLocked" = 0 
                WHERE "SysPackageId" IN (SELECT "Id" FROM "SysPackage" WHERE "UId" = '{package_uid}');
                
                UPDATE "SysPackageSchemaData" 
                SET "IsChanged" = 0, "IsLocked" = 0 
                WHERE "SysPackageId" IN (SELECT "Id" FROM "SysPackage" WHERE "UId" = '{package_uid}');
                
                UPDATE "SysPackageSqlScript" 
                SET "IsChanged" = 0, "IsLocked" = 0 
                WHERE "SysPackageId" IN (SELECT "Id" FROM "SysPackage" WHERE "UId" = '{package_uid}');    
                            
                UPDATE "SysPackageReferenceAssembly" 
                SET "IsChanged" = 0, "IsLocked" = 0 
                WHERE "SysPackageId" IN (SELECT "Id" FROM "SysPackage" WHERE "UId" = '{package_uid}');
                "#
            ),
            CrtDbType::Oracle | CrtDbType::Postgres => format!(
                r#"UPDATE "SysSchema" 
                SET "IsChanged" = False, "IsLocked" = False 
                WHERE "SysPackageId" IN (SELECT "Id" FROM "SysPackage" WHERE "UId" = '{package_uid}');
                
                UPDATE "SysPackageSchemaData" 
                SET "IsChanged" = False, "IsLocked" = False 
                WHERE "SysPackageId" IN (SELECT "Id" FROM "SysPackage" WHERE "UId" = '{package_uid}');
                
                UPDATE "SysPackageSqlScript" 
                SET "IsChanged" = False, "IsLocked" = False 
                WHERE "SysPackageId" IN (SELECT "Id" FROM "SysPackage" WHERE "UId" = '{package_uid}');
                
                UPDATE "SysPackageReferenceAssembly" 
                SET "IsChanged" = False, "IsLocked" = False 
                WHERE "SysPackageId" IN (SELECT "Id" FROM "SysPackage" WHERE "UId" = '{package_uid}');
                "#
            ),
        };

        Ok(self.0.sql(&query)?.rows_affected)
    }

    pub fn delete_package_localizations(
        &self,
        package_uid: &str,
    ) -> Result<u64, CrtClientGenericError> {
        let query = match self.0.db_type()? {
            _ => &format!(
                r#"DELETE FROM "SysLocalizableValue" 
                WHERE "SysPackageId" IN (
                    SELECT "Id" FROM "SysPackage" WHERE "UId" = '{package_uid}'
                );
                
                DELETE FROM "SysPackageResourceChecksum" 
                WHERE "SysPackageId" IN (
                    SELECT "Id" FROM "SysPackage" WHERE "UId" = '{package_uid}'
                );
                
                DELETE FROM "SysPackageDataLcz" WHERE "SysPackageSchemaDataId" IN (
                    SELECT "Id" FROM "SysPackageSchemaData" WHERE "SysPackageId" IN (
                        SELECT "Id" FROM "SysPackage" WHERE "UId" = '{package_uid}'
                    )
                );
                
                DELETE FROM "SysPackageSchemaData" 
                WHERE "SysPackageId" IN (
                    SELECT "Id" FROM "SysPackage" WHERE "UId" = '{package_uid}'
                );
                "#
            ),
        };

        Ok(self.0.sql(query)?.rows_affected)
    }

    pub fn reset_schema_content(&self, package_uid: &str) -> Result<u64, CrtClientGenericError> {
        let query = match self.0.db_type()? {
            _ => &format!(
                r#"DELETE FROM "SysSchemaContent" WHERE "SysSchemaId" IN (
                    SELECT "Id" FROM "SysSchema" WHERE "SysPackageId" IN (
                        SELECT "Id" FROM "SysPackage" WHERE "UId" = '{package_uid}'
                    )
                )
                
                UPDATE "SysSchema"
                SET "Checksum" = '',
                "MetaData" = NULL,
                "Descriptor" = NULL,
                "CreatedOn" = NULL,
                "ModifiedById" = NULL,
                "CreatedById" = NULL,
                "ModifiedOn" = NULL,
                "ClientContentModifiedOn" = NULL
                WHERE "SysPackageId" IN (
                    SELECT "Id" FROM "SysPackage" WHERE "UId" = '{package_uid}'
                )
                "#
            ),
        };

        Ok(self.0.sql(query)?.rows_affected)
    }

    pub fn lock_package(&self, package_name: &str) -> Result<u64, CrtClientGenericError> {
        let query = match self.0.db_type()? {
            CrtDbType::MsSql => &format!(
                r#"UPDATE "SysPackage" 
                SET "InstallType" = 1, "IsLocked" = 0, "IsChanged" = 0
                WHERE "Name" = '{package_name}';
                "#
            ),
            CrtDbType::Oracle | CrtDbType::Postgres => &format!(
                r#"UPDATE "SysPackage" 
                SET "InstallType" = 1, "IsLocked" = False, "IsChanged" = False
                WHERE "Name" = '{package_name}';
                "#
            ),
        };

        Ok(self.0.sql(query)?.rows_affected)
    }

    pub fn unlock_package(&self, package_name: &str) -> Result<u64, CrtClientGenericError> {
        let query = match self.0.db_type()? {
            CrtDbType::MsSql => &format!(
                r#"UPDATE "SysPackage" 
                SET "InstallType" = 0, "IsLocked" = 1, "IsChanged" = 1
                WHERE "Name" = '{package_name}';
                "#
            ),
            CrtDbType::Oracle | CrtDbType::Postgres => &format!(
                r#"UPDATE "SysPackage" 
                SET "InstallType" = 0, "IsLocked" = True, "IsChanged" = True
                WHERE "Name" = '{package_name}';
                "#
            ),
        };

        Ok(self.0.sql(query)?.rows_affected)
    }
}
