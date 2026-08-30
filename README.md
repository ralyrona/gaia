# Github Automated Installation Application
GAIA is commandline installation application with a similar syntax to Apt. It is designed to install applications from GitHub, the world's largest open-source file sharing website. It can download executable files from the releases of any github repository*!<br>
<sub>*Excluding repositories in the [Limitations section](https://github.com/ralyrona/gaia#Limitations).</sub>
# How to Install
Note that GAIA is currently compatible only with Debian-based GNU/Linux.
1. Install cURL with `sudo apt install curl`.
2. Download GAIA from one of the releases.
3. Execute it from the commandline (if you are trying to use a commandline installation application we would hope that you know how to do that) ***as root*** with the argument `setup` (e.g. `sudo ./gaia setup`). It will then install itself into your `/usr/local/bin` directory and allow you to use by running the `gaia` command.
# How to Use
As mentioned earlier, GAIA has a syntax very similar to Apt's syntax. **There are differences**, however, and some may simply not use Apt, so we have added this documentation. (Note that you can access similar documentation via the application by running `gaia help`).
## Syntax
Syntax: `gaia <subcommand> <arguments>`<br>
## Subcommands
`install`     install an application *(requires root priveleges)*<br>
`remove`      remove an application *(requires root priveleges)*<br>
`override`    same as install, but does not throw an error if it has to overwrite files, can be used to update applications *(requires root priveleges)*<br>
`help`        show help information, or show more detailed information about a specific subcommand
### `install` and `override`
Syntax: `sudo gaia <install or override> <account name>/<repository name>`<br>
The install subcommand is used to install applications. To specify the application to install, you must enter the name of the account which owns the repository, followed by a forward slash (/), followed by the name of the repository. For example, you could run `sudo gaia install ralyrona/gump` to install GUMP, my image editor, or you could run `sudo gaia install fish-shell/fish-shell` to install Fish, the Friendly Interactive Shell. Please note that to install an application, the targeted repository must have at least one release and its latest release must have at least one asset that is not source code. The install subcommand will fail if it tries to overwrite files.<br>
The override subcommand is similar to install, except it will not fail if forced to overwrite files. It can be used to update applications, as it will install the latest release and overwrite outdated files.
### `remove`
Syntax: `sudo gaia remove <account name>/<repository name>`<br>
Remove removes applications and any configuration data that *GAIA* created for them. Note that some data may remain, such as extremely out of date files whose references were removed during an override. GAIA will also not remove configuration data that the application itself created.
# Limitations
1. GAIA requires for the repository to have at least one release.
2. The repository must have a release marked as latest.
3. The latest release must have at least one non-source-code asset.
# Compilation from Source
GAIA requires OpenSSL to function. OpenSSL can be installed with this command:
`sudo apt install libssl-dev`
Then it can be compiled with `cargo build` and `cargo build --release`.
# Issues
1. There are issues with creating the proper configuration data for applications installed from file archives (e.g. zip, tar, etc.)
2. If you use the `override` command to update an existing application, then the existence of certain files may become entirely unknown by the configuration data, making them impossible to remove automatically
