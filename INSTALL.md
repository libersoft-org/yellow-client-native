# Yellow Client - native application - installation

## 1. Download the latest version of this software

These are the installation instructions of this software for the different operating systems.

Log in as "root" (Linux) or "Administrator" (Windows) on your PC and run the following commands to download the necessary dependencies and the latest version of this software from GitHub:

### Windows

1. Download the [**.NET SDK**](https://dotnet.microsoft.com/en-us/download/dotnet/thank-you/sdk-8.0.401-windows-x64-installer) and install it.
2. Download the latest version of [**Yellow Native Client**](https://github.com/libersoft-org/yellow-client-native/archive/refs/heads/main.zip) wrapper source codes.
3. Unpack the content of "**main.zip**" file.
4. Run **Command Prompt** in Windows and navigate to the folder where you unpacked this software.
5. Run "**build.bat**" file.
6. Run **Yellow.exe** in "**build**" subdirectory.

### Debian / Ubuntu Linux

```sh
wget https://packages.microsoft.com/config/debian/12/packages-microsoft-prod.deb -O packages-microsoft-prod.deb
dpkg -i packages-microsoft-prod.deb
rm packages-microsoft-prod.deb
apt update
apt -y upgrade
apt -y install git 
git clone https://github.com/libersoft-org/yellow-client-native.git
cd yellow-client-native/src/
./build.sh
cd build
dotnet Yellow.exe
```
