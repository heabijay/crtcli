# crtcli

Creatio Command Line Interface (crtcli) — A command-line tool for interacting with Creatio and Creatio packages, focusing on enhancing the developer experience.

A tiny [clio](https://github.com/Advance-Technologies-Foundation/clio) utility alternative.

> crtcli is under researching & development. The CLI interface may change, so exercise caution when using it in scripts and remember to check for updates.

## Installation

### via pre-build binaries

Download the archive from [release page](https://github.com/heabijay/crtcli/releases) for your platform, extract it, and run executable from terminal.

To use crtcli from anywhere, add the directory containing the executable to your system's PATH environment variable.

### via cURL (Linux & macOS)

```shell
curl -sfL https://raw.githubusercontent.com/heabijay/crtcli/main/install-unix.sh | sh
```

### via PowerShell (Windows)

```powershell
iwr -useb https://raw.githubusercontent.com/heabijay/crtcli/main/install-windows.ps1 | iex
```


## Commands / Features

- [x] [app](#app)
    - [x] [compile](#app-compile)
    - [x] [flush-redis](#app-flush-redis)
    - [x] [fs](#app-fs)
      - [x] [check](#app-fs-check)
      - [x] [pull](#app-fs-pull)
      - [x] [push](#app-fs-push)
    - [x] [install-log](#app-install-log)
    - [x] [pkg](#app-pkg)
      - [x] [compile](#app-pkg-compile)
      - [x] [download](#app-pkg-download)
      - [x] [fs](#app-pkg-fs)
        - [x] [pull](#app-pkg-fs-pull)
        - [x] [push](#app-pkg-fs-push)
      - [x] [install](#app-pkg-install)
      - [x] [get-uid](#app-pkg-get-uid)
      - [x] [lock](#app-pkg-lock)
      - [x] [pull](#app-pkg-pull)
      - [x] [push](#app-pkg-push)
      - [x] [unlock](#app-pkg-unlock)
    - [x] [pkgs](#app-pkgs)
    - [x] [restart](#app-restart)
    - [x] [request](#app-request)
    - [x] [sql](#app-sql)
- [x] [pkg](#pkg)
    - [x] [apply](#pkg-apply)
    - [x] [pack](#pkg-pack)
    - [x] [unpack](#pkg-unpack)
    - [x] [unpack-all](#pkg-unpack-all)


### [Root Command]

**Options:**

- `--help | -h` — Print help for any command.

- `--version | -V` — Print crtcli version.

- `--completions <SHELL>` — Generate shell completions config for your shell. This config should be added to your shell configuration file or folder. Currently, this completions config is getting generated using the 'clap_complete' crate.

    Possible values: bash, elvish, fish, powershell, zsh

    Defaults: trying to autodetect


### app

Commands to interact with Creatio application instance.

Please check [dotenv (.env) files](#dotenv-env-files) for simplified commands usage.

**Arguments:**

- `<URL>` (required) (env: `CRTCLI_APP_URL`) —  The base URL of Creatio instance.

- `<USERNAME>` (required) (env: `CRTCLI_APP_USERNAME`) — Creatio Username.

- `<PASSWORD>` (required) (env: `CRTCLI_APP_PASSWORD`) — Creatio Password.

**Options:**

- `--insecure | -i` (env: `CRTCLI_APP_INSECURE`) — Bypass SSL certificate verification. Use with caution, primarily for development or testing environments.

- `--net-framework` (env: `CRTCLI_APP_NETFRAMEWORK`) — Use .NET Framework (IIS) Creatio compatibility 

  By default, crtcli primary uses .NET Core / .NET (Kestrel) API routes to operate with remote. However, some features like "app restart" works by different API routes in both platforms.


### app compile

Compiles the Creatio application (equivalent to the "Build" or "Rebuild" action in the Creatio Configuration section).

**Options:**

- `--force-rebuild | -f` — Perform a rebuild instead of a standard build.

- `--restart | -r` — Restart the Creatio application after successful compilation.

**Examples:**

- `crtcli app https://localhost:5000 Supervisor Supervisor -i compile` — Compiles the Creatio instance at insecure https://localhost:5000.

- `crtcli app compile -fr` — Compiles the Creatio instance specified by the $CRTCLI_APP_URL environment variable, using a forced rebuild and restarting afterward.


### app flush-redis

Clears the Redis cache associated with the Creatio instance.

**Examples:**

- `crtcli app https://localhost:5000 Supervisor Supervisor -i flush-redis` — Flushes the Redis cache for the insecure Creatio instance at https://localhost:5000.

- `crtcli app flush-redis` — Flushes Redis cache in Creatio '$CRTCLI_APP_URL'.


### app fs

Commands for interacting with Creatio's File System Development (FSD) mode.


### app fs check

Print if File System Development mode is enabled for the Creatio instance.

**Examples:**

- `crtcli app https://localhost:5000 Supervisor Supervisor -i fs check` — Check is File System Development mode status for the insecure Creatio 'https://localhost:5000'. (True/False)

- `crtcli app fs check` — Check is File System Development mode enabled in Creatio '$CRTCLI_APP_URL'.


### app fs pull

Unload packages from Creatio database into filesystem.

**Arguments:**

- `[PACKAGES]` — A space-separated or comma-separated list of package names to pull. If omitted, all* packages from database will be pulled.

  _\* Creatio pulls only unlocked packages that you can modify in Creatio Configuration._

**Examples:**

- `crtcli app https://localhost:5000 Supervisor Supervisor -i fs pull` — Pulls all packages from database into filesystem at insecure Creatio 'https://localhost:5000'.

- `crtcli app fs pull UsrPackage` — Pulls single package 'UsrPackage' from database into filesystem in Creatio '$CRTCLI_APP_URL'.

- `crtcli app fs pull UsrPackage UsrPackage2` | `crtcli app fs pull UsrPackage,UsrPackage2` — Pulls packages 'UsrPackage' and 'UsrPackage2' from database into filesystem in Creatio '$CRTCLI_APP_URL'.


### app fs push

Load packages from filesystem into Creatio database.

**Arguments:**

- `[PACKAGES]` — A space-separated or comma-separated list of package names to push. If omitted, all* packages from filesystem will be pushed.

  _\* Creatio pushes only unlocked packages that you can modify in Creatio Configuration._

**Examples:**

- `crtcli app https://localhost:5000 Supervisor Supervisor -i fs push` — Pushes all packages from filesystem into database at insecure Creatio 'https://localhost:5000'.

- `crtcli app fs push UsrPackage` — Pushes single package 'UsrPackage' from filesystem into database in Creatio '$CRTCLI_APP_URL'.

- `crtcli app fs push UsrPackage UsrPackage2` | `crtcli app fs push UsrPackage,UsrPackage2` — Pushes packages 'UsrPackage' and 'UsrPackage2' from filesystem into database in Creatio '$CRTCLI_APP_URL'.


### app install-log

Print last package installation log.

**Examples:**

- `crtcli app https://localhost:5000 Supervisor Supervisor -i install-log` — Gets last package installation log at insecure Creatio 'https://localhost:5000'.

- `crtcli app install-log` — Gets last package installation log in Creatio '$CRTCLI_APP_URL'.


### app pkg

Commands to manipulate with packages in Creatio.

Many of these commands will attempt to infer the target package name from the current working directory if it's a package folder (contains a descriptor.json file).


### app pkg compile

Compiles a specific package within the Creatio instance.

**Arguments:**

- `[PACKAGE_NAME]` — Name of package to compile.

  Defaults: If omitted, crtcli will try to determine the package name from the current directory (by looking for descriptor.json).

**Options:**

- `--restart | -r` — Restart the Creatio application after successful package compilation.

**Examples:**

For example current folder is '/Creatio_8.1.5.2176/Terrasoft.Configuration/Pkg/UsrPackage' which is package folder.

- `crtcli app https://localhost:5000 Supervisor Supervisor -i pkg compile UsrCustomPkg` — Compiles package 'UsrCustomPkg' at insecure Creatio 'https://localhost:5000'.

- `crtcli app pkg compile -r` — Compiles the UsrPackage (inferred from the current directory) in the Creatio instance defined by $CRTCLI_APP_URL and restarts the application.


### app pkg download

Downloads one or more packages from the Creatio instance as a zip archive.

**Arguments:**

- `[PACKAGES]` — A space-separated or comma-separated list of package names to download. 

  Defaults: If omitted, crtcli will try to determine the package name from the current directory (by looking for descriptor.json).

**Options:**

- `--output-folder | -f <FOLDER>` — Directory where the downloaded package archive will be saved.

  Defaults: Current directory.

- `--output-filename | -n <FILENAME>` — Name of the output zip file.

  Defaults: Autogenerated — `PackageName_YYYY-MM-DD_HH-mm-ss.zip` for single package and `Packages_YYYY-MM-DD_HH-mm-ss.zip` for multiple packages.

**Examples:**

For example current folder is '/Creatio_8.1.5.2176/Terrasoft.Configuration/Pkg/UsrPackage' which is package folder.

- `crtcli app https://localhost:5000 Supervisor Supervisor -i pkg download UsrCustomPkg` — Downloads package 'UsrCustomPkg' from insecure Creatio 'https://localhost:5000' to current directory.

- `crtcli app pkg download -f /backups -n MyPackage.zip` — Downloads package 'UsrPackage' (cause current folder is this package) from Creatio '$CRTCLI_APP_URL' to '/backups' folder with filename 'MyPackage.zip'.

- `crtcli app pkg download UsrPkg1 UsrPkg2` | `crtcli app pkg download UsrPkg1,UsrPkg2` — Downloads packages 'UsrPkg1' & 'UsrPkg2' from Creatio '$CRTCLI_APP_URL' to current folder.


### app pkg fs

Commands/aliases to simplify manipulate with package insides File System Development mode (FSD) location.

They are designed to be used from within a package directory located under the Creatio file system packages path, for example:

`<Creatio_Dir>/Terrasoft.Configuration/Pkg/<Package_Name>`

And, of course, in this scenario, your Creatio should have File System Development mode enabled.


### app pkg fs pull

Unload package in current folder from Creatio database into filesystem and applies any configured transforms (see [pkg apply](#pkg-apply)).

Alternative to:

```shell
crtcli app fs pull "{package_name}" # {package_name} is inferred from the current directory 
crtcli pkg apply .
```

**Options:**

- `--package-folder <PACKAGE_FOLDER>` — Package folder where package is already pulled previously.

  Defaults: Current directory

  Sample: <Creatio_Dir>/Terrasoft.Configuration/Pkg/<Package_Name>

And here you can use transforms from [pkg apply](#pkg-apply) command.

\* Check [package.crtcli.toml](#packagecrtclitoml) to configure default apply transforms.

**Examples:**

For example current folder is '/Creatio_8.1.5.2176/Terrasoft.Configuration/Pkg/UsrPackage' which is package folder inside in Creatio (FSD mode enabled).

- `crtcli app https://localhost:5000 Supervisor Supervisor -i pkg fs pull` — Pulls package 'UsrPackage' to filesystem from Creatio (using FSD) at insecure 'https://localhost:5000' and tries to apply configured transforms to it (for example from package.crtcli.toml file if exists).

- `crtcli app pkg fs pull -S true` — Pulls package 'UsrPackage' from Creatio (using FSD) on '$CRTCLI_APP_URL' and applies sorting transform.


### app pkg fs push

Load package  in current folder from filesystem into Creatio database and optionally compiles it.

Alternative to:

```shell
crtcli app fs push "{package_name}" # {package_name} is inferred from the current directory
crtcli app pkg compile "{package_name}" -r
```

**Options:**

- `--package-folder <PACKAGE_FOLDER>` — Package folder where package is already pulled previously.

  Defaults: Current directory

  Sample: <Creatio_Dir>/Terrasoft.Configuration/Pkg/<Package_Name>

- `--compile-package-after-push | -c` — Compile package after successful push.

- `--restart-app-after-compile | -r` — Restart the Creatio application after successful compilation.

**Examples:**

For example current folder is '/Creatio_8.1.5.2176/Terrasoft.Configuration/Pkg/UsrPackage' which is package folder inside in Creatio (FSD mode enabled).

- `crtcli app https://localhost:5000 Supervisor Supervisor -i pkg fs push` — Pushes package 'UsrPackage' from filesystem to Creatio (using FSD) at insecure 'https://localhost:5000'.

- `crtcli app pkg fs push -cr` — Pushes package 'UsrPackage' from filesystem to Creatio (using FSD) on '$CRTCLI_APP_URL', compiles it after successfully push, and restarts Creatio application if compilation was successful.


### app pkg install

Installs a package archive (.zip or .gz) into the Creatio instance.

**Arguments:**

- `<FILEPATH>` (required) — Path to the package archive file.

**Options:**

- `--restart | -r` — Restart the Creatio application after successful installation.

- `--force | -f` (sql) — Overrides changed schemas in the database. Use this if you've modified schemas in an unlocked package within Creatio, and the installing process is preventing updates to those schemas.

  Under the hood, this option executes the following SQL script before package installation to mark all package schemas as unchanged:

  ```sql
  UPDATE "SysSchema" 
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
  ```

- `--force-and-clear-localizations | -F` (sql) — Same as -f but also clears localization data. Use this if you want to remove outdated or unwanted localization strings. — _This options makes resources diffs less trashy during pull/push workflow._

  Under the hood, this option executes the following SQL script before package installation:

  ```sql
  -- SQL script from --force (-f) command here --
  
  -- Then:
  
  DELETE FROM "SysLocalizableValue" 
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
  ```
  
- `--clear-schemas-content` (sql) — Clears existing schema content and checksums before installation. Use this if schema content (e.g., C# code) is not updating correctly from the package.

  Under the hood, this option executes the following SQL script before package installation:

  ```sql
  DELETE FROM "SysSchemaContent" WHERE "SysSchemaId" IN (
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
  ```
  
- `--disable-install-log-pooling` — Disables the display of the installation log.

  
\* (sql) — Requires an installed sql runner package in Creatio that is supported by crtcli. Please check [app sql](#app-sql) command documentation. 


**Examples:**

- `crtcli app https://localhost:5000 Supervisor Supervisor -i pkg install /repo/UsrPackage-latest.zip` — Installs package archive '/repo/UsrPackage-latest.zip' at insecure Creatio 'https://localhost:5000'.

- `crtcli app pkg install UsrPackage.gz -fr` — Executes SQL to mark all 'UsrPackage' schemas as not changed, installs package 'UsrPackage.gz' in Creatio '$CRTCLI_APP_URL' and restarts it after successful installation.

- `crtcli app pkg install UsrPackage.gz -Fr` — Executes SQL to mark all 'UsrPackage' schemas as not changed, clears all localization data of 'UsrPackage' schemas, installs package 'UsrPackage.gz' in Creatio '$CRTCLI_APP_URL' and restarts it after successful installation.


### app pkg get-uid

Print installed package information by Package UId.

**Arguments:**

- `<PACKAGE_UID>` (required) — UId of the package.

**Options:**

- `--json` — Display the output in JSON format.

**Examples:**

- `crtcli app https://localhost:5000 Supervisor Supervisor -i pkg get-uid ae8519c2-2aac-4a00-aa61-b0ffaac99ea3` — Prints information about package 'ae8519c2-2aac-4a00-aa61-b0ffaac99ea3' at insecure Creatio 'https://localhost:5000'.

  stdout:
  ```
  ActionsDashboard (ae8519c2-2aac-4a00-aa61-b0ffaac99ea3)
  | Id: 96adf8f9-652d-4382-843c-d91ff737478c
  | Created on: 2020-05-27T12:09:53.095
  | Modified on: 2022-10-04T15:37:06.000
  | Maintainer: Terrasoft
  | Type: 0
  ```

- `crtcli app pkg get-uid ae8519c2-2aac-4a00-aa61-b0ffaac99ea3 --json` — Prints information about package 'ae8519c2-2aac-4a00-aa61-b0ffaac99ea3' in Creatio '$CRTCLI_APP_URL' in JSON format.

  stdout:
  ```json
  {"id":"96adf8f9-652d-4382-843c-d91ff737478c","uId":"ae8519c2-2aac-4a00-aa61-b0ffaac99ea3","name":"ActionsDashboard","type":0,"maintainer":"Terrasoft","createdOn":"2020-05-27T12:09:53.095","modifiedOn":"2022-10-04T15:37:06.000"}
  ```


### app pkg lock

Execute SQL to make package locked if it is unlocked in Creatio.

```sql
UPDATE "SysPackage" 
SET "InstallType" = 1, "IsLocked" = False, "IsChanged" = False
WHERE "Name" = '{package_name}';   
```

\* Requires an installed sql runner package in Creatio that is supported by crtcli. Please check [app sql](#app-sql) command documentation.

**Arguments:**

- `[PACKAGE_NAME]` — Package name to lock.

  Defaults: Tries to determine package name from current folder as package folder. (From file ./descriptor.json)

**Examples:**

For example current folder is '/Creatio_8.1.5.2176/Terrasoft.Configuration/Pkg/UsrPackage' which is package folder.

- `crtcli app https://localhost:5000 Supervisor Supervisor -i pkg lock UsrCustomPackage` — Locks package 'UsrCustomPackage' at insecure Creatio 'https://localhost:5000'.

- `crtcli app pkg lock` — Locks package 'UsrPackage' (cause current folder is this package) in Creatio '$CRTCLI_APP_URL'.


### app pkg pull

Downloads a package from Creatio, unpacks it to a destination folder, and applies configured transforms. This is a more efficient alternative to manually downloading, unpacking, and applying transforms.

Alternative to:

```shell
crtcli app pkg download "{package_name}" --output-filename "tmp-pkg.zip"
crtcli pkg unpack "tmp-pkg.zip" . --merge
crtcli pkg apply .
rm "tmp-pkg.zip"
```

but faster due to in memory processing, merging only changes and more feature-rich.


**Options:**

- `--package | -p <PACKAGE_NAME>` — Package name to pull.

  Defaults: Tries to determine package name from destination folder. (From file ./descriptor.json)

- `--destination-folder | -d <DESTINATION_FOLDER>` — Destination folder where the package files will be unpacked using merge.

  Defaults: Current directory

And here you can use transforms from [pkg apply](#pkg-apply) command.

\* Check [package.crtcli.toml](#packagecrtclitoml) to configure default apply transforms.

**Examples:**

For example current folder is '/Creatio_8.1.5.2176/Terrasoft.Configuration/Pkg/UsrPackage' which is package folder.

- `crtcli app https://localhost:5000 Supervisor Supervisor -i pkg pull -p UsrCustomPackage -d /repos/UsrCustomPackage -S true` — Downloads package 'UsrCustomPackage' from insecure Creatio 'https://localhost:5000' and unpacks it into /repos/UsrCustomPackage folder with sorting transform.

- `crtcli app pkg pull` — Downloads package 'UsrPackage' (cause current folder is this package) from Creatio '$CRTCLI_APP_URL' and unpacks it into current folder using merge with default applied transforms.


### app pkg push

Packs a package from a source folder and installs it into the Creatio instance. This is a more efficient alternative to manually packing and installing.

Alternative to:

```shell
crtcli pkg pack . --format gzip --output-filename "tmp-package.gz"
crtcli app install "tmp-package.gz"
rm "tmp-package.gz"
```

but it works faster due to in memory processing and merging only changes and also has additional features.

**Options:**

- `--source-folder | -s <SOURCE_FOLDERS>` — Folder containing the package to be packed and installed. You can specify multiple source folders to install several packages at once.

  Defaults: Current directory

And here you can use options from [app pkg install](#app-pkg-install) command like --restart, --force, ...

**Examples:**

For example current folder is '/Creatio_8.1.5.2176/Terrasoft.Configuration/Pkg/UsrPackage' which is package folder.

- `crtcli app https://localhost:5000 Supervisor Supervisor -i pkg push -s /repos/UsrCustomPackage` — Packs and installs package 'UsrCustomPackage' into insecure Creatio 'https://localhost:5000'.

- `crtcli app pkg push -Fr` — Packs and installs package 'UsrPackage' (cause current folder is this package) into Creatio '$CRTCLI_APP_URL' with executing sql scripts to mark package schemas as unchanged, schema localization cleanup and restarts application after install. 

- `crtcli app pkg push -s /repos/UsrCustomPackage1 -s /repos/UsrCustomPackage2` — Packs and installs packages 'UsrCustomPackage1' and 'UsrCustomPackage2' into Creatio '$CRTCLI_APP_URL' at once. 


### app pkg unlock

Execute SQL to make package unlocked if it is locked in Creatio.

```sql
UPDATE "SysPackage"
SET "InstallType" = 0, "IsLocked" = True, "IsChanged" = True
WHERE "Name" = '{package_name}';
```

\* Requires an installed sql runner package in Creatio that is supported by crtcli. Please check [app sql](#app-sql) command documentation.

\** Note: To be able to edit the unlocked package, ensure that the Maintainer in kagethe pac matches the Maintainer system setting in Creatio. You may need to log out and log back in after change the Maintainer system setting.

**Arguments:**

- `[PACKAGE_NAME]` — Name of the package to unlock.

  Defaults: Tries to determine package name from current folder as package folder. (From file ./descriptor.json)

**Examples:**

For example current folder is '/Creatio_8.1.5.2176/Terrasoft.Configuration/Pkg/UsrPackage' which is package folder.

- `crtcli app https://localhost:5000 Supervisor Supervisor -i pkg unlock UsrCustomPackage` — Unlocks package 'UsrCustomPackage' at insecure Creatio 'https://localhost:5000'.

- `crtcli app pkg unlock` — Unlocks package 'UsrPackage' (cause current folder is this package) in Creatio '$CRTCLI_APP_URL'.


### app pkgs

Lists the installed packages in the Creatio instance.

**Options:**

- `--json` — Display the output in JSON format.

**Examples:**

- `crtcli app https://localhost:5000 Supervisor Supervisor -i pkgs` — Prints list of installed packages at insecure Creatio 'https://localhost:5000'.

  stdout:
  ```
  ActionsDashboard (UId: ae8519c2-2aac-4a00-aa61-b0ffaac99ea3)
  AnalyticsDashboard (UId: 02abeaad-7dcc-4f15-86c9-cd6090362e82)
  Approval (UId: 1eefea8c-efe3-53d9-6397-3aac9cc9e785)
  ...
  ```

- `crtcli app pkgs --json` — Prints list of installed packages in Creatio '$CRTCLI_APP_URL' in JSON format.

  stdout:
  ```json
  [{"uId":"ae8519c2-2aac-4a00-aa61-b0ffaac99ea3","name":"ActionsDashboard"},{"uId":"02abeaad-7dcc-4f15-86c9-cd6090362e82","name":"AnalyticsDashboard"},{"uId":"1eefea8c-efe3-53d9-6397-3aac9cc9e785","name":"Approval"},...
  ```


### app restart

Restarts the Creatio application.

Important: If your Creatio instance is running on .NET Framework (IIS), you must use the --net-framework flag with the app command. Otherwise, the restart will not be executed, and you won't receive an error.

**Examples:**

- `crtcli app https://localhost:5000 Supervisor Supervisor -i restart` — Restarts Creatio application at insecure 'https://localhost:5000'.

- `crtcli app restart` — Restarts Creatio application '$CRTCLI_APP_URL'.


### app request

Sends authenticated HTTP requests to the Creatio instance, similar to curl.

**Arguments:**

- `<METHOD>` (required) — HTTP method (e.g., GET, POST, PUT, DELETE, etc.).

- `<URL>` (required) — URL to request (can be absolute or relative to the Creatio base URL).

**Options:**

- `--anonymous | -a` — Send the request without authentication.

- `--data | -d <DATA>` — Request body data (for methods like POST).

- `--data-stdin | -D` — Read the request body data from standard input. Use a double Enter to signal the end of input.

- `--header | -H <HEADER>` — Add a custom header to the request (format: Key: Value). The default Content-Type is application/json.

- `--output | -o <FILE>` — Save the response body to a file.

**Examples:**

- `crtcli app https://localhost:5000 Supervisor Supervisor -i request GET 0/rest/UsrService/UsrMethod` — Sends an authenticated GET request to 'https://localhost:5000/0/rest/UsrService/UsrMethod' at insecure Creatio.

- `crtcli app request POST 0/rest/UsrService/UsrPostMethod -d '{"request": "test"}'` — Sends an authenticated POST request to '0/rest/UsrService/UsrPostMethod' to Creatio '$CRTCLI_APP_URL' with body '{"request": "test"}'.

- `crtcli app request POST 0/rest/UsrService/UsrPostMethod -D` — Sends an authenticated POST request to '0/rest/UsrService/UsrPostMethod' to Creatio '$CRTCLI_APP_URL' with body read from stdin.

  stdin & stdout:
  ```shell
  Please enter request data (body) below: 
  -=-=- -=-=- -=-=- -=-=- -=-=-
  
  {"request":"test"}
  
  -=-=- -=-=- -=-=- -=-=- -=-=-
  
  Status: 404 Not Found
  Content: 0 bytes read
  ```

- `crtcli app request GET 0/ServiceModel/PublicService.svc/UsrPubMethod -a -H "X-Access-Token: 123"` — Sends an anonymous GET request to '0/ServiceModel/PublicService.svc/UsrPubMethod' to Creatio '$CRTCLI_APP_URL' with custom header 'X-Access-Token: 123'.


### app sql

Executes SQL queries in the Creatio database using a supported SQL runner package installed in Creatio.

_Beta: this command is still under development._

**Supported SQL packages:**

- [cliogate](https://raw.githubusercontent.com/Advance-Technologies-Foundation/clio/refs/heads/master/cliogate.gz)
- SqlConsole

**Arguments:**

- `[SQL]` — SQL query to execute.

  Defaults: If omitted and the --file option is not used, crtcli will prompt you to enter the query from standard input (use a double Enter to finish).

**Options:**

- `--file | -f <FILE>` — Read the SQL query from a file.

- `--runner | -r <RUNNER>` — Specify the SQL runner to use.

  Possible values: cliogate, sql-console

  Defaults: Autodetect

- `--json` — Display the results in JSON format.

**Examples:**

- `crtcli app https://localhost:5000 Supervisor Supervisor -i sql 'SELECT COUNT(*) FROM "SysPackage"'` — Executes SQL query 'SELECT COUNT(*) FROM "SysPackage"' at insecure Creatio 'https://localhost:5000' with automatically detected sql runner.

  stdout:
  ```json
  [
    {
      "count": 359
    }
  ]
  ```
  
- `crtcli app sql'` — Executes SQL query from stdin in Creatio '$CRTCLI_APP_URL' with automatically detected sql runner.

  stdin & stdout:
  ```shell
  Please enter SQL query below: 
  -=-=- -=-=- -=-=- -=-=- -=-=-
  
  select count(*) from "Contact"
  
  -=-=- -=-=- -=-=- -=-=- -=-=-
  
  [
    {
      "count": 18
    }
  ]
  ```
  
- `crtcli app sql -r sql-console -f query.sql'` — Executes SQL query from file 'query.sql' in Creatio '$CRTCLI_APP_URL' with sql-console runner.


### pkg

Commands for working with Creatio package files (.zip, .gz) or package folders locally, without interacting with a Creatio instance.


### pkg apply

Applies transformations to the contents of a package folder. 

This is useful for standardizing package structure, cleaning up localization files, and improving version control diffs, etc.

**Arguments:**

- `<PACKAGE_FOLDER>` (required) — Path to the package folder.

**Options:**

- `--file | -f <FILE>` — Apply transforms only to a specific file within the package folder. The path should be relative to the package folder or absolute (but still within the package folder). Useful for pre-commit git hooks.

**Transforms (also options):**

- `--apply-sorting | -S <true/false>` — Sorts the contents of specific files within the package. This is useful when you work with any VCS cause it prevents you from check some inconsistent diffs.

    _Affects:_
  - descriptor.json
  - Data/**/*.json
  - Data/**/Localization/*.json
  - Files/*.csproj

- `--apply-localization-cleanup | -L <EXCEPT-LOCALIZATIONS>` —  Removes localization files except for the specified cultures (comma-separated list).

    _Affects_:
  - Data/**/Localization/data.*.json
  - Resources/**/resource.*.xml

**Examples:**

- `crtcli pkg apply . --apply-sorting true` — Applies sorting transform to the current package folder.

- `crtcli pkg apply /Creatio_8.1.5.2176/Terrasoft.Configuration/Pkg/UsrPackage -S true -L 'en-US,uk-UA'` — Applies sorting and localization cleanup transforms to package '/Creatio_8.1.5.2176/Terrasoft.Configuration/Pkg/UsrPackage'. Localization cleanup deletes all localization files in this folder except for 'en-US' and 'uk-UA' cultures.


### pkg pack

Creates a package archive (.zip or .gz) from a package folder.

Included Paths:

- `Assemblies/*`
- `Data/*`
- `Files/*`
- `Resources/*`
- `Schemas/*`
- `SqlScripts/*`
- `descriptor.json`

Excluded: Hidden folders and files (names starting with .).

**Arguments:**

- `<PACKAGE_FOLDER>` (required) — Source folder containing the package files to be packaged.

**Options:**

- `--output-folder | -f <FOLDER>` — Destination folder where the output package archive will be saved.

  Defaults: Current Directory

- `--output-filename | -n <FILENAME>` — Filename of the output package archive file.
    
    Defaults: Autogenerated — `PackageName_YYYY-MM-DD_HH-mm-ss.zip` for zip format and `PackageName.gz` for gzip.

- `--format <FORMAT>` — Archive format.

    Possible values: gzip, zip    

    Defaults: zip

- `--compression <COMPRESSION>` — Compression level (fast, normal, best)

    Possible values: fast, normal, best

    Defaults: fast

**Examples:**

For example current folder is '/Creatio_8.1.5.2176/Terrasoft.Configuration/Pkg/UsrPackage' which is package folder.

- `crtcli pkg pack .` — Packs current folder as package and outputs package file 'UsrPackage_2024-12-01_21-00-00.zip' to current directory.

- `crtcli pkg pack /Creatio_8.1.5.2176/Terrasoft.Configuration/Pkg/UsrPackage2 --format gzip --compression best` — Packs folder '/Creatio_8.1.5.2176/Terrasoft.Configuration/Pkg/UsrPackage2' as package and outputs package file 'UsrPackage2.gz' to current directory.

- `crtcli pkg pack . -f /backups/` — Packs current folder as package and outputs package file 'UsrPackage_2024-12-01_21-00-00.zip' to '/backups/' folder.

- `crtcli pkg pack . -f /backups/ -n 'UsrPackage-latest.zip'` — Packs current folder as package and outputs package file 'UsrPackage-latest.zip' to '/backups/' folder. If file already exists — it will be replaced.


### pkg unpack

Extract a single package from a package archive (.zip or .gz). To extract multiple packages from a zip archive, use [pkg unpack-all](#pkg-unpack-all).

**Arguments:**

- `<PACKAGE_FILEPATH>` (required) — Path to the package archive file.

- `[DESTINATION_FOLDER]` — Destination folder where the extracted package files will be saved.

    Defaults: '{Filename without extension}' folder in current folder. If this folder already exists — creates a new one with suffix '_1' and so on.

**Options:**

- `--package | -p <PACKAGE_NAME>` — If the archive is a zip file containing multiple packages, specify the name of the package to extract.

- `--merge | -m` — If destination_folder already exists you will receive error about this by default. However, you can use this merge option to extract to same exist folder with overwriting only different files.

And here you can use transforms from [pkg apply](#pkg-apply) command.

**Examples:**

- `crtcli pkg unpack UsrPackage_2024-12-01_21-00-00.zip` — Extracts single package from 'UsrPackage_2024-12-01_21-00-00.zip' file to folder './UsrPackage_2024-12-01_21-00-00/'

- `crtcli pkg unpack UsrPackage.gz /repos/UsrPackage -mS true` — Extracts single package from 'UsrPackage.gz' file to folder './repos/UsrPackage' with merging and sorting transform.

- `crtcli pkg unpack UsrMultiplePackages_2024-12-01_21-00-00.zip UsrPackageSources -p UsrPackage` — Extracts single package 'UsrPackage' (file UsrPackage.gz) from 'UsrMultiplePackages_2024-12-01_21-00-00.zip' file to folder './UsrPackageSources/'.


### pkg unpack-all

Extract all packages from a zip archive.

**Arguments:**

- `<PACKAGE_FILEPATH>` (required) — Path to the zip package archive file.

- `[DESTINATION_FOLDER]` — Destination folder where all extracted package files will be saved.

  Defaults: '{Filename without extension}' folder in current folder. If this folder already exists — creates a new one with suffix '_1' and so on.

**Options:**

- `--merge | -m` — If destination_folder already exists you will receive error about this by default. However, you can use this merge option to extract to same exist folder with overwriting only different files.

And here you can use transforms from [pkg apply](#pkg-apply) command.

**Examples:**

For example, file 'MyPackage.zip' contains one 'UsrPackage' package, and file 'MyMultiplePackages.zip' contains 'UsrPackage1' package and 'UsrPackage2' package.

- `crtcli pkg unpack-all MyPackage.zip` — Extracts packages from 'MyPackage.zip' file to folder './MyPackage/'. 

    The output folder structure will be:
  - MyPackage/
    - UsrPackage/
      - ...

- `crtcli pkg unpack-all MyMultiplePackages.zip /repos/ -mL 'en-US'` — Extracts packages from 'MyMultiplePackages.zip' file to folder '/repos/' with merging and localization cleanup transform except 'en-US' culture.

    The output folder structure will be:
  - /repos/
    - UsrPackage1/
      - ...
    - UsrPackage2/
      - ...


## Config files


### dotenv (.env) files

crtcli supports .env files for storing environment variables, simplifying command usage by avoiding repetitive argument entry.

Locations: '.env', '.crtcli.env' in current directory or any parent folders.

**Variables:**

- `CRTCLI_APP_URL` — The base URL of Creatio instance by default.

- `CRTCLI_APP_USERNAME` — Creatio username by default.

- `CRTCLI_APP_PASSWORD` — Creatio password by default.

- `CRTCLI_APP_INSECURE` — Set to 'true' to disable SSL certificate validation by default.

- `CRTCLI_APP_NETFRAMEWORK` — Set to 'true' if your Creatio instance is running on .NET Framework (IIS) by default.

**Examples:**

For example, current folder is '/Creatio_8.1.5.2176/Terrasoft.Configuration/Pkg/UsrPackage' which is package folder inside in Creatio. 

You could have a .env file at /Creatio_8.1.5.2176/.env with the following content:

```
CRTCLI_APP_URL="https://localhost:88"
CRTCLI_APP_USERNAME="Supervisor"
CRTCLI_APP_PASSWORD="Supervisor@1"
CRTCLI_APP_INSECURE="true"
```

Now, from within /Creatio_8.1.5.2176/Terrasoft.Configuration/Pkg/UsrPackage or any of its parent directories, you can run commands like:

- `crtcli app pkgs` — This will list the packages from https://localhost:88 because $CRTCLI_APP_URL is defined in the .env file.

- `crtcli app ...` —  Any other app command will similarly use the environment variables from the .env file.


### package.crtcli.toml

The package.crtcli.toml file is an optional configuration file that allows you to customize crtcli's behavior for a specific package.

Location: ./package.crtcli.toml within the package folder.

Check [toml syntax here](https://toml.io/en/v1.0.0).

**Parameters:**

- `apply.sorting = <true/false>` — Enable/disable sorting transform by default in [pkg apply](#pkg-apply) command.

- `apply.localization_cleanup = <except-localizations>` — Enable/disable localization cleanup transform by default in [pkg apply](#pkg-apply) command.

**Examples:**

1. For example, current folder is '/Creatio_8.1.5.2176/Terrasoft.Configuration/Pkg/UsrPackage' which is package folder inside in Creatio.

    You could have a package.crtcli.toml file in this directory with the following content:
  
    ```toml
    [apply]
    sorting = true
    localization_cleanup = ["en-US", "uk-UA"]
    ```

   The package folder structure would look like:
    - /Creatio_8.1.5.2176/Terrasoft.Configuration/Pkg/UsrPackage
      - Data/*
      - Resources/*
      - Schemas/*
      - ...
      - descriptor.json
      - package.crtcli.toml

    With this configuration:

    - `crtcli pkg apply .` — Will apply both sorting and localization cleanup (keeping only en-US and uk-UA cultures) because they are enabled in package.crtcli.toml.

    - `crtcli app pkg pull` —  Will download UsrPackage, unpack it, and apply the sorting and localization cleanup transforms defined in package.crtcli.toml.

    - `crtcli app pkg fs pull` — Will download UsrPackage to the file system and apply the sorting and localization cleanup transforms defined in package.crtcli.toml.

---

Stay tuned!
