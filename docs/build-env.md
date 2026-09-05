# Build Environment

The build environment tries to create an environment which is as 'clean' as possible. This will help with more stable and reproducible builds. This is done by stripping and adding some environment variables.

## Adjusted PATH
The build environment creates an adjusted `PATH`. The `PATH` will contain the bin directories of all (build) dependencies. If on Unix, it will also include the standard Unix system bin paths, which are:
- `/usr/bin`
- `/bin`
- `/usr/sbin`
- `/sbin`

When on Windows the standard Windows system bin paths are added to the `PATH`, which are:
- `C:\Windows`
- `C:\Windows\system32`
- `C:\Windows\System32\Wbem`
- `C:\Windows\System32\WindowsPowerShell\v1.0`

## Pkgconfig path
Packit looks for the presence of a `pkgconfig` directory inside of the `share` and `lib` directories of the dependencies of a package. These paths are then added to the `PKG_CONFIG_PATH` to make them available. It also adds `/usr/lib/pkgconfig` which is a macOS specific path.

## Non-symlinked dependencies
Some packages are not symlinked, so they cannot be found by cmake in the build process. For that reason, dependencies which are not symlinked are added to the `CMAKE_PREFIX_PATH`. The same applies to the `ACLOCAL_PATH`, with the small difference that build dependencies are also added in this case.

## Requirement environment
Some requirements need a certain environment setup. This is different for each requirement. All requirement environments are explained here.

### Windows MSVC
Windows MSVC is a requirement for some packages when building on Windows. If MSVC is installed then the installation is automatically detected by Packit if necessary. The following environment variables are then created:
| Key                    | Explanation                                                                  |
| ---------------------- | ---------------------------------------------------------------------------- |
| `PACKIT_VS_PATH`       | Contains the path to Visual Studio.                                          |
| `PACKIT_VCVARSALL`     | Contains the path to the `vcvarsall.bat` script.                             |
| `PACKIT_VCVARSALL_ARCH`| Contains the correct architecture string with which to call `vcvarsall.bat`. |
| `PACKIT_MSVC_VERSION`  | Contains the MSVC version.                                                   |

The following command should be used to initialize the MSVC environment in a build script:<br>
`call "%PACKIT_VCVARSALL%" %PACKIT_VCVARSALL_ARCH%`

## Extra environment variables
In addition to the variables listed above, there are some extra environment variables. The following variables are added:
| Key            | Value                    | Explanation                                                                                 |
| -------------- | ------------------------ | ------------------------------------------------------------------------------------------- |
| `TZ`           | `UTC0`                   | Ensures that the timezone is the same across all builds.                                    |
| `M4`           | `<package-prefix/bin/m4` | This variable is only added if it is a build dependency and ensures that `m4` is available. |
| `PERL`         | `/usr/bin/perl`          | Makes Perl available. (macOS only)                                                          |
| `ZERO_AR_DATE` | `1`                      | Ensures that there are no arbitrary timestamps are in builds. (macOS only)                  |
