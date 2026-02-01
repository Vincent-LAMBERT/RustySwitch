const { invoke } = window.__TAURI__.core;

window.addEventListener("DOMContentLoaded", () => {
  setupApp();
  document.getElementById("save-btn").addEventListener('click', storeActiveSinks); 
});

// Store the shortcut
let userShortcut = null;

// Listen for key presses on the input field
document.getElementById('shortcutInput').addEventListener('keydown', function(e) {
    e.preventDefault(); // Prevent default behavior (e.g., typing in the input)
    const key = e.key;
    const ctrlKey = e.ctrlKey;
    const shiftKey = e.shiftKey;
    const altKey = e.altKey;

    // Format the shortcut (e.g., "Ctrl+Shift+S")
    let shortcut = [];
    if (ctrlKey) shortcut.push('Ctrl');
    if (shiftKey) shortcut.push('Shift');
    if (altKey) shortcut.push('Alt');
    shortcut.push(key);
    userShortcut = shortcut.join('+');

    this.value = userShortcut; // Show the shortcut in the input
});

let server;
async function setupApp () {
  await invoke("get_sound_server").then((x) => server = x);
  console.log(server)
  getAudioOutputs();
}


let sinks;
let activeSink
// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
async function getAudioOutputs() {
  await invoke("get_audio_outputs", {soundServer: server}).then((sinks_and_active) => updateSinks(sinks_and_active));

  updateSelect("mainSelect");
  updateSelect("secondSelect");
}

function updateSinks(sinks_and_active) {
  sinks = sinks_and_active[0]
  activeSink = sinks_and_active[1]
}

function updateSelect(id) {
  let selectEl = document.getElementById(id);
  // The transmission from Rust to Javascript adds an unused null value
  for (var i = 0; i<sinks.length; i++){
    var opt = document.createElement('option');
    opt.value = sinks[i];
    opt.innerHTML = sinks[i];
    selectEl.appendChild(opt);
  }
  updateSelected(selectEl)
}

function updateSelected(selectEl, active=null) {
  if (active!=null) {
    selectEl.value = active;
  } else {
    let takenValues = [];
    let selectElements = document.getElementsByTagName("select");
    
    for (var i = 0; i < selectElements.length; i++) {
      if (selectElements[i].id != selectEl.id) {
        takenValues.push(selectElements[i].value);
      }
    }

    let availableSinks = sinks.filter(n => !takenValues.includes(n))
    if (availableSinks.length > 0) {
      selectEl.value = availableSinks[0];
    }
  }
}

function storeActiveSinks() {
  let takenValues = [];
  let selectElements = document.getElementsByTagName("select");
  
  for (var i = 0; i < selectElements.length; i++) {
    takenValues.push(selectElements[i].value);
  }
  console.log("storeActiveSinks");

  invoke("set_key_in_store", {key: "takenValues", value: takenValues});
}
