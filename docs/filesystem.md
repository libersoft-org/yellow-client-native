
# yellow client:
accounts:
```
 function clickDownload(e) {
  console.log('clickDownload', e);
  let blob = new Blob([JSON.stringify($accounts_config, null, 2)], { type: 'application/json' });
  console.log('blob', blob);
  let url = URL.createObjectURL(blob);
  let a = document.createElement('a');
  a.href = url;
  a.download = 'accounts_' + new Date().toISOString().replace('T', ' ').replace('Z', '').replace(/\.\d+/, '') + '.json';
  a.click();
  ...
```

file upload
```
<input type="file" id="fileInput" bind:this={elFileInput} ...
elFileInput.click();...
```
