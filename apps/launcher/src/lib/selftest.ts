// Самопроверка интерфейса для CI: открывает и закрывает панели и пишет
// размеры окна после каждого шага. Включается только переменной CUBER_SELFTEST.

import { selftestEnabled, selftestLog } from "./api";
import type { LauncherState } from "./state/launcher.svelte";

const wait = (ms: number) => new Promise((r) => setTimeout(r, ms));

export async function runSelftest(app: LauncherState) {
  if (!(await selftestEnabled())) return;
  await wait(2500);
  await selftestLog("start");
  for (let i = 1; i <= 10; i++) {
    app.togglePanel("friends");
    await wait(800);
    await selftestLog(`${i}:friends`);
    app.togglePanel("settings");
    await wait(800);
    await selftestLog(`${i}:settings`);
    app.closePanels();
    await wait(800);
    await selftestLog(`${i}:closed`);
    app.profileMenuOpen = true;
    await wait(600);
    await selftestLog(`${i}:profiles`);
    app.profileMenuOpen = false;
    await wait(600);
  }
  app.launchError = "Проверочная ошибка";
  await wait(800);
  await selftestLog("error-card");
  app.launchError = null;
  await wait(800);
  await selftestLog("DONE");
}
