// Логика окна установщика. Без сборщика: Tauri отдаёт API через window.__TAURI__.

const tauri = window.__TAURI__;
const invoke = (cmd, args) =>
  tauri ? tauri.core.invoke(cmd, args) : Promise.reject(new Error("нет Tauri"));
const win = tauri ? tauri.window.getCurrentWindow() : null;

const $ = (id) => document.getElementById(id);
const sleep = (ms) => new Promise((r) => setTimeout(r, ms));

let dir = "";
let isUpdate = false;

function show(id) {
  for (const s of document.querySelectorAll("section")) s.hidden = s.id !== id;
}

function toggle(button) {
  const on = button.getAttribute("aria-checked") !== "true";
  button.setAttribute("aria-checked", String(on));
  button.querySelector(".track").classList.toggle("on", on);
}

function errorText(e) {
  return typeof e === "string" ? e : e?.message || String(e);
}

// Окно: перетаскивание за шапку, свернуть, закрыть.
$("drag").addEventListener("mousedown", (e) => {
  if (e.button === 0 && !e.target.closest("button")) win?.startDragging();
});
$("min").onclick = () => win?.minimize();
$("close").onclick = () => win?.close();
$("finish").onclick = () => win?.close();
$("keep").onclick = () => win?.close();
$("desktop").onclick = () => toggle($("desktop"));
$("wipe").onclick = () => toggle($("wipe"));

$("change").onclick = async () => {
  const picked = await invoke("pick_folder").catch(() => null);
  if (picked) {
    dir = picked;
    $("dir").textContent = dir;
    $("dir").title = dir;
  }
};

$("install").onclick = async () => {
  $("i-error").hidden = true;
  show("s-progress");
  $("p-title").textContent = isUpdate ? "Обновляю…" : "Устанавливаю…";
  const bar = $("bar");
  const steps = ["Копирую файлы", "Создаю ярлыки", "Регистрирую в Windows"];
  // Установка занимает доли секунды — показываем шаги, чтобы было понятно, что произошло.
  const run = invoke("install", {
    dir,
    desktopShortcut: $("desktop").getAttribute("aria-checked") === "true",
  });
  for (let i = 0; i < steps.length; i++) {
    $("p-step").textContent = steps[i];
    bar.style.width = `${((i + 1) / steps.length) * 90}%`;
    await sleep(350);
  }
  try {
    await run;
    bar.style.width = "100%";
    await sleep(250);
    $("d-title").textContent = isUpdate ? "Cuber обновлён" : "Cuber установлен";
    show("s-done");
  } catch (e) {
    show("s-install");
    $("i-error").textContent = errorText(e);
    $("i-error").hidden = false;
  }
};

$("run").onclick = () =>
  invoke("launch", { dir }).catch((e) => {
    $("d-lead").textContent = errorText(e);
  });

$("remove").onclick = async () => {
  $("u-error").hidden = true;
  $("remove").disabled = true;
  try {
    await invoke("uninstall", { removeData: $("wipe").getAttribute("aria-checked") === "true" });
    $("d-title").textContent = "Cuber удалён";
    $("d-lead").textContent = "Спасибо, что играли! Возвращайтесь.";
    $("run").hidden = true;
    show("s-done");
  } catch (e) {
    $("u-error").textContent = errorText(e);
    $("u-error").hidden = false;
  } finally {
    $("remove").disabled = false;
  }
};

(async () => {
  let info;
  try {
    info = await invoke("info");
  } catch (e) {
    $("u-reason").textContent = errorText(e);
    show("s-unsupported");
    return;
  }
  $("ver").textContent = `v${info.version}`;

  if (info.mode === "uninstall") {
    document.title = "Удаление Cuber";
    show("s-uninstall");
    return;
  }
  if (!info.supported || !info.hasPayload) {
    $("u-reason").textContent = !info.supported
      ? "Этот установщик работает только в Windows."
      : "Установщик собран без лаунчера внутри.";
    show("s-unsupported");
    return;
  }

  isUpdate = !!info.installedDir;
  dir = info.installedDir || info.defaultDir;
  $("dir").textContent = dir;
  $("dir").title = dir;
  if (isUpdate) {
    $("i-title").textContent = "Обновление Cuber";
    $("i-lead").textContent = "Cuber уже установлен — поставлю новую версию. Миры и профили сохранятся.";
    $("install").textContent = "Обновить";
    $("change").hidden = true;
  }
  show("s-install");
})();
