#!/bin/sh

dotnet publish -c Release -r linux-x64 -o build --self-contained /p:PublishSingleFile=true /p:IncludeAllContentForSelfExtract=true
