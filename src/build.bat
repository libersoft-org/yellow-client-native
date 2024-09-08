md build
dotnet publish -c Release -r win-x64  -o build --self-contained /p:PublishSingleFile=true /p:IncludeAllContentForSelfExtract=true