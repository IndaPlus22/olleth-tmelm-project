
const invoke = window.__TAURI__.invoke;

(async () => {
  //let user = await create_user();
  await handleFileUpload();
})();


async function handleFileUpload() {
  let fileInput = document.getElementById("file-input");
  let fileList = document.getElementById("files-list");
  let numOfFiles = document.getElementById("num-of-files");
  fileInput.addEventListener("change", async () => {
    fileList.innerHTML = "";
    numOfFiles.textContent = `${fileInput.files.length} Files Selected`;
    for (i of fileInput.files) {
      let reader = new FileReader();
      let listItem = document.createElement("li");
      let fileName = i.name;
      let fileSize = (i.size / 1024).toFixed(1);
      listItem.innerHTML = `<p id="file-name">${fileName}</p><p id="file-size">${fileSize}MB</p><div id="spinner"></div>`;
      if (fileSize >= 1024) {
        fileSize = (fileSize / 1024).toFixed(1);
        listItem.innerHTML = `<p id="file-name">${fileName}</p><p id="file-size">${fileSize}MB</p><div id="spinner"></div>`;
      }
      fileList.appendChild(listItem);

      // const bytes = await readFileAsBytes(i);
      // console.log(bytes);

      // let result = Boolean(await invoke("upload_video", {user: user, filename: fileName, bytes: String(bytes)}));

      // if (result) {
      //   document.querySelector('#spinner').remove();
      //   listItem.innerHTML += `<div id="checkmark"></div>`;
      // } else {
      //   document.querySelector('#spinner').remove();
      //   listItem.innerHTML += `<div id="xmark"></div>`;
      // }
    }
  });
}

async function username(user) {
  await invoke("username", {user: user});
}

async function create_user() {
  // Get the overlay and popup elements
  const overlay = document.getElementById('overlay');
  const popup = document.getElementById('popup');

  // Display the popup and overlay
  popup.classList.remove('hidden');
  overlay.classList.remove('hidden');

  let user = await invoke("create_user", { path: "../secrets" });
  //let cloneuser = Object.assign({}, user);

  // Hide the popup and overlay after the verification is completed
  popup.classList.add('hidden');
  overlay.classList.add('hidden');
  
  return user;
}

function readFileAsBytes(file) {
  return new Promise((resolve, reject) => {
    const reader = new FileReader();
    reader.onload = () => {
      const arrayBuffer = reader.result;
      const bytes = new Uint8Array(arrayBuffer);
      resolve(bytes);
    };
    reader.onerror = () => {
      reject(reader.error);
    };
    reader.readAsArrayBuffer(file);
  });
}